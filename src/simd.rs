//! SIMD optimizations for agent-md.
//!
//! Provides platform-accelerated primitives for:
//! - Newline counting (`count_newlines`)
//! - Two-byte set membership checking (`has_byte2`)
//! - Consecutive space detection (`has_consecutive_spaces`)
//! - ASCII case-insensitive substring searching (`contains_ascii_case_insensitive`)
//!
//! Includes AArch64 (NEON), x86_64 (AVX2 / SSE2 with runtime detection),
//! and scalar fallback implementations.

// ============================================================================
// 1. Newline Counting
// ============================================================================

/// Scalar reference for newline counting.
pub fn count_newlines_scalar(data: &[u8]) -> usize {
	data.iter().filter(|&&b| b == b'\n').count()
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn count_newlines_neon(data: &[u8]) -> usize {
	// SAFETY:
	// - NEON is baseline on AArch64.
	// - `vld1q_u8` supports unaligned loads.
	// - Loop increments `i` by 16 as long as `i + 16 <= len`, so pointers stay in bounds.
	// - Batch accumulator sums at most 255 matches per lane, preventing u8 overflow.
	use core::arch::aarch64::*;

	let len = data.len();
	let ptr = data.as_ptr();
	let needle = vdupq_n_u8(b'\n');
	let one = vdupq_n_u8(1);
	let mut i = 0;
	let mut total = 0usize;

	while i + 16 <= len {
		let mut acc = vdupq_n_u8(0);
		let batch_end = (i + 4080).min(len - (len - i) % 16);
		while i < batch_end {
			let chunk = vld1q_u8(ptr.add(i));
			let mask = vceqq_u8(chunk, needle);
			let ones = vandq_u8(mask, one);
			acc = vaddq_u8(acc, ones);
			i += 16;
		}
		total += vaddlvq_u8(acc) as usize;
	}

	while i < len {
		if *data.get_unchecked(i) == b'\n' {
			total += 1;
		}
		i += 1;
	}

	total
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn count_newlines_avx2(data: &[u8]) -> usize {
	// SAFETY:
	// - Caller verifies `is_x86_feature_detected!("avx2")`.
	// - `_mm256_loadu_si256` supports unaligned loads.
	// - Pointers remain within bounds `0..len`.
	use core::arch::x86_64::*;

	let len = data.len();
	let ptr = data.as_ptr();
	let needle = _mm256_set1_epi8(b'\n' as i8);
	let mut i = 0;
	let mut total = 0usize;

	while i + 32 <= len {
		let chunk = _mm256_loadu_si256(ptr.add(i) as *const __m256i);
		let cmp = _mm256_cmpeq_epi8(chunk, needle);
		let mask = _mm256_movemask_epi8(cmp) as u32;
		total += mask.count_ones() as usize;
		i += 32;
	}

	while i < len {
		if *data.get_unchecked(i) == b'\n' {
			total += 1;
		}
		i += 1;
	}

	total
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn count_newlines_sse2(data: &[u8]) -> usize {
	// SAFETY:
	// - SSE2 is baseline on x86_64.
	// - `_mm_loadu_si128` supports unaligned loads.
	// - Pointers remain within bounds `0..len`.
	use core::arch::x86_64::*;

	let len = data.len();
	let ptr = data.as_ptr();
	let needle = _mm_set1_epi8(b'\n' as i8);
	let mut i = 0;
	let mut total = 0usize;

	while i + 16 <= len {
		let chunk = _mm_loadu_si128(ptr.add(i) as *const __m128i);
		let cmp = _mm_cmpeq_epi8(chunk, needle);
		let mask = _mm_movemask_epi8(cmp) as u32;
		total += mask.count_ones() as usize;
		i += 16;
	}

	while i < len {
		if *data.get_unchecked(i) == b'\n' {
			total += 1;
		}
		i += 1;
	}

	total
}

/// Count newlines in a byte slice using SIMD acceleration.
pub fn count_newlines(data: &[u8]) -> usize {
	#[cfg(target_arch = "aarch64")]
	{
		return unsafe { count_newlines_neon(data) };
	}

	#[cfg(target_arch = "x86_64")]
	{
		if is_x86_feature_detected!("avx2") {
			return unsafe { count_newlines_avx2(data) };
		}
		return unsafe { count_newlines_sse2(data) };
	}

	#[allow(unreachable_code)]
	count_newlines_scalar(data)
}

// ============================================================================
// 2. Two-byte Set Membership
// ============================================================================

/// Scalar reference for checking if `data` contains either `b1` or `b2`.
pub fn has_byte2_scalar(data: &[u8], b1: u8, b2: u8) -> bool {
	data.iter().any(|&b| b == b1 || b == b2)
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn has_byte2_neon(data: &[u8], b1: u8, b2: u8) -> bool {
	// SAFETY:
	// - NEON is baseline on AArch64.
	// - `vld1q_u8` supports unaligned loads.
	// - Pointers remain within bounds `0..len`.
	use core::arch::aarch64::*;

	let len = data.len();
	let ptr = data.as_ptr();
	let v1 = vdupq_n_u8(b1);
	let v2 = vdupq_n_u8(b2);
	let mut i = 0;

	while i + 16 <= len {
		let chunk = vld1q_u8(ptr.add(i));
		let cmp1 = vceqq_u8(chunk, v1);
		let cmp2 = vceqq_u8(chunk, v2);
		let matched = vorrq_u8(cmp1, cmp2);
		if vmaxvq_u8(matched) != 0 {
			return true;
		}
		i += 16;
	}

	while i < len {
		let b = *data.get_unchecked(i);
		if b == b1 || b == b2 {
			return true;
		}
		i += 1;
	}

	false
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn has_byte2_avx2(data: &[u8], b1: u8, b2: u8) -> bool {
	// SAFETY:
	// - Caller verifies `is_x86_feature_detected!("avx2")`.
	// - `_mm256_loadu_si256` supports unaligned loads.
	// - Pointers remain within bounds `0..len`.
	use core::arch::x86_64::*;

	let len = data.len();
	let ptr = data.as_ptr();
	let v1 = _mm256_set1_epi8(b1 as i8);
	let v2 = _mm256_set1_epi8(b2 as i8);
	let mut i = 0;

	while i + 32 <= len {
		let chunk = _mm256_loadu_si256(ptr.add(i) as *const __m256i);
		let cmp1 = _mm256_cmpeq_epi8(chunk, v1);
		let cmp2 = _mm256_cmpeq_epi8(chunk, v2);
		let matched = _mm256_or_si256(cmp1, cmp2);
		if _mm256_movemask_epi8(matched) != 0 {
			return true;
		}
		i += 32;
	}

	while i < len {
		let b = *data.get_unchecked(i);
		if b == b1 || b == b2 {
			return true;
		}
		i += 1;
	}

	false
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn has_byte2_sse2(data: &[u8], b1: u8, b2: u8) -> bool {
	// SAFETY:
	// - SSE2 is baseline on x86_64.
	// - `_mm_loadu_si128` supports unaligned loads.
	// - Pointers remain within bounds `0..len`.
	use core::arch::x86_64::*;

	let len = data.len();
	let ptr = data.as_ptr();
	let v1 = _mm_set1_epi8(b1 as i8);
	let v2 = _mm_set1_epi8(b2 as i8);
	let mut i = 0;

	while i + 16 <= len {
		let chunk = _mm_loadu_si128(ptr.add(i) as *const __m128i);
		let cmp1 = _mm_cmpeq_epi8(chunk, v1);
		let cmp2 = _mm_cmpeq_epi8(chunk, v2);
		let matched = _mm_or_si128(cmp1, cmp2);
		if _mm_movemask_epi8(matched) != 0 {
			return true;
		}
		i += 16;
	}

	while i < len {
		let b = *data.get_unchecked(i);
		if b == b1 || b == b2 {
			return true;
		}
		i += 1;
	}

	false
}

/// Check if `data` contains either `b1` or `b2` using SIMD acceleration.
pub fn has_byte2(data: &[u8], b1: u8, b2: u8) -> bool {
	#[cfg(target_arch = "aarch64")]
	{
		return unsafe { has_byte2_neon(data, b1, b2) };
	}

	#[cfg(target_arch = "x86_64")]
	{
		if is_x86_feature_detected!("avx2") {
			return unsafe { has_byte2_avx2(data, b1, b2) };
		}
		return unsafe { has_byte2_sse2(data, b1, b2) };
	}

	#[allow(unreachable_code)]
	has_byte2_scalar(data, b1, b2)
}

// ============================================================================
// 3. Consecutive Spaces Detection
// ============================================================================

/// Scalar reference for detecting two consecutive spaces (`b"  "`).
pub fn has_consecutive_spaces_scalar(data: &[u8]) -> bool {
	data.windows(2).any(|w| w == b"  ")
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn has_consecutive_spaces_neon(data: &[u8]) -> bool {
	// SAFETY:
	// - NEON is baseline on AArch64.
	// - `vld1q_u8` supports unaligned loads.
	// - Pointers remain within bounds `0..len`.
	use core::arch::aarch64::*;

	let len = data.len();
	if len < 2 {
		return false;
	}
	let ptr = data.as_ptr();
	let space_v = vdupq_n_u8(b' ');
	let zero = vdupq_n_u8(0);
	let mut i = 0;

	while i + 16 <= len {
		let chunk = vld1q_u8(ptr.add(i));
		let cmp = vceqq_u8(chunk, space_v);
		let shifted = vextq_u8(cmp, zero, 1);
		let pairs = vandq_u8(cmp, shifted);
		if vmaxvq_u8(pairs) != 0 {
			return true;
		}
		i += 15;
	}

	while i + 1 < len {
		if *data.get_unchecked(i) == b' ' && *data.get_unchecked(i + 1) == b' ' {
			return true;
		}
		i += 1;
	}

	false
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn has_consecutive_spaces_sse2(data: &[u8]) -> bool {
	// SAFETY:
	// - SSE2 is baseline on x86_64.
	// - `_mm_loadu_si128` supports unaligned loads.
	// - Pointers remain within bounds `0..len`.
	use core::arch::x86_64::*;

	let len = data.len();
	if len < 2 {
		return false;
	}
	let ptr = data.as_ptr();
	let space_v = _mm_set1_epi8(b' ' as i8);
	let mut i = 0;

	while i + 16 <= len {
		let chunk = _mm_loadu_si128(ptr.add(i) as *const __m128i);
		let cmp = _mm_cmpeq_epi8(chunk, space_v);
		let mask = _mm_movemask_epi8(cmp) as u32;
		if (mask & (mask >> 1)) != 0 {
			return true;
		}
		i += 15;
	}

	while i + 1 < len {
		if *data.get_unchecked(i) == b' ' && *data.get_unchecked(i + 1) == b' ' {
			return true;
		}
		i += 1;
	}

	false
}

/// Check if `data` contains at least two consecutive spaces (`b"  "`).
pub fn has_consecutive_spaces(data: &[u8]) -> bool {
	#[cfg(target_arch = "aarch64")]
	{
		return unsafe { has_consecutive_spaces_neon(data) };
	}

	#[cfg(target_arch = "x86_64")]
	{
		return unsafe { has_consecutive_spaces_sse2(data) };
	}

	#[allow(unreachable_code)]
	has_consecutive_spaces_scalar(data)
}

// ============================================================================
// 4. Case-Insensitive ASCII Substring Search
// ============================================================================

/// Scalar reference for case-insensitive ASCII substring search.
pub fn contains_ascii_case_insensitive_scalar(haystack: &str, needle: &str) -> bool {
	if needle.is_empty() {
		return true;
	}
	if haystack.len() < needle.len() {
		return false;
	}
	let h_bytes = haystack.as_bytes();
	let n_bytes = needle.as_bytes();
	let n_len = n_bytes.len();

	h_bytes
		.windows(n_len)
		.any(|window| window.eq_ignore_ascii_case(n_bytes))
}

/// Case-insensitive substring search for ASCII needles without allocating lowercase strings.
///
/// Uses SIMD two-byte search for the first byte of `needle` (both lowercase and uppercase)
/// to quickly skip chunks of `haystack` that cannot contain a match.
pub fn contains_ascii_case_insensitive(haystack: &str, needle: &str) -> bool {
	if needle.is_empty() {
		return true;
	}
	if haystack.len() < needle.len() {
		return false;
	}

	let h_bytes = haystack.as_bytes();
	let n_bytes = needle.as_bytes();
	let n_len = n_bytes.len();

	// Quick check: if needle is 1 byte, delegate directly to SIMD has_byte2
	if n_len == 1 {
		let b0 = n_bytes[0];
		return has_byte2(h_bytes, b0.to_ascii_lowercase(), b0.to_ascii_uppercase());
	}

	let b0_lower = n_bytes[0].to_ascii_lowercase();
	let b0_upper = n_bytes[0].to_ascii_uppercase();

	let mut offset = 0;
	let limit = h_bytes.len().saturating_sub(n_len - 1);

	while offset < limit {
		// Use SIMD has_byte2 to test if the next slice contains the first character
		let slice_end = (offset + 64).min(limit);
		if !has_byte2(&h_bytes[offset..slice_end], b0_lower, b0_upper) {
			offset = slice_end;
			continue;
		}

		// Found candidate byte within window, check individual positions
		for pos in offset..slice_end {
			let b = h_bytes[pos];
			if (b == b0_lower || b == b0_upper)
				&& h_bytes[pos..pos + n_len].eq_ignore_ascii_case(n_bytes)
			{
				return true;
			}
		}
		offset = slice_end;
	}

	false
}

// ============================================================================
// Unit and Property Tests
// ============================================================================

#[cfg(test)]
mod tests {
	use super::*;
	use proptest::prelude::*;

	#[test]
	fn test_count_newlines_basic() {
		assert_eq!(count_newlines(b""), 0);
		assert_eq!(count_newlines(b"hello"), 0);
		assert_eq!(count_newlines(b"hello\n"), 1);
		assert_eq!(count_newlines(b"\n\n\n"), 3);
		assert_eq!(
			count_newlines(b"a\nb\nc\nd\ne\nf\ng\nh\ni\nj\nk\nl\nm\nn\no\np\nq\n"),
			17
		);
	}

	#[test]
	fn test_has_byte2_basic() {
		assert!(!has_byte2(b"", b'*', b'_'));
		assert!(!has_byte2(b"hello world", b'*', b'_'));
		assert!(has_byte2(b"hello * world", b'*', b'_'));
		assert!(has_byte2(b"hello _ world", b'*', b'_'));
		assert!(has_byte2(b"12345678901234567890*", b'*', b'_'));
	}

	#[test]
	fn test_has_consecutive_spaces_basic() {
		assert!(!has_consecutive_spaces(b""));
		assert!(!has_consecutive_spaces(b" "));
		assert!(!has_consecutive_spaces(b"a b c d e f"));
		assert!(has_consecutive_spaces(b"  "));
		assert!(has_consecutive_spaces(b"a  b"));
		assert!(has_consecutive_spaces(b"0123456789012345  "));
	}

	#[test]
	fn test_contains_ascii_case_insensitive_basic() {
		assert!(contains_ascii_case_insensitive("", ""));
		assert!(contains_ascii_case_insensitive("Hello", ""));
		assert!(!contains_ascii_case_insensitive("", "hello"));
		assert!(contains_ascii_case_insensitive("Hello World", "hello"));
		assert!(contains_ascii_case_insensitive("Hello World", "WORLD"));
		assert!(contains_ascii_case_insensitive("Hello World", "o w"));
		assert!(!contains_ascii_case_insensitive("Hello World", "earth"));
	}

	#[test]
	fn test_simd_vs_scalar_performance() {
		use std::time::Instant;

		let mut sample = Vec::with_capacity(500_000);
		for i in 0..500_000 {
			if i % 80 == 0 {
				sample.push(b'\n');
			} else if i % 17 == 0 {
				sample.push(b'*');
			} else if i % 23 == 0 {
				sample.push(b' ');
			} else {
				sample.push(b'a' + (i % 26) as u8);
			}
		}

		let iters = 50;
		let t0 = Instant::now();
		let mut sum_simd = 0;
		for _ in 0..iters {
			sum_simd += count_newlines(&sample);
		}
		let d_simd = t0.elapsed();

		let t1 = Instant::now();
		let mut sum_scalar = 0;
		for _ in 0..iters {
			sum_scalar += count_newlines_scalar(&sample);
		}
		let d_scalar = t1.elapsed();

		assert_eq!(sum_simd, sum_scalar);
		eprintln!(
			"Newline count 500KB: SIMD {:?}, Scalar {:?}, Speedup: {:.2}x",
			d_simd,
			d_scalar,
			d_scalar.as_secs_f64() / d_simd.as_secs_f64()
		);
	}

	proptest! {
		#[test]
		fn proptest_count_newlines_matches_scalar(
			data in prop::collection::vec(any::<u8>(), 0..5000)
		) {
			let simd_result = count_newlines(&data);
			let scalar_result = count_newlines_scalar(&data);
			prop_assert_eq!(simd_result, scalar_result);
		}

		#[test]
		fn proptest_has_byte2_matches_scalar(
			data in prop::collection::vec(any::<u8>(), 0..2000),
			b1 in any::<u8>(),
			b2 in any::<u8>(),
		) {
			let simd_result = has_byte2(&data, b1, b2);
			let scalar_result = has_byte2_scalar(&data, b1, b2);
			prop_assert_eq!(simd_result, scalar_result);
		}

		#[test]
		fn proptest_has_consecutive_spaces_matches_scalar(
			data in prop::collection::vec(any::<u8>(), 0..2000)
		) {
			let simd_result = has_consecutive_spaces(&data);
			let scalar_result = has_consecutive_spaces_scalar(&data);
			prop_assert_eq!(simd_result, scalar_result);
		}

		#[test]
		fn proptest_contains_ascii_case_insensitive_matches_scalar(
			haystack in "[a-zA-Z0-9 _-]{0,200}",
			needle in "[a-zA-Z0-9 _-]{0,20}",
		) {
			let simd_result = contains_ascii_case_insensitive(&haystack, &needle);
			let scalar_result = contains_ascii_case_insensitive_scalar(&haystack, &needle);
			prop_assert_eq!(simd_result, scalar_result);
		}
	}
}
