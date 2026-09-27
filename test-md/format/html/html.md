# Format HTML

## Rule: Remove useless spaces, tabs, or newlines inside HTML tags

HTML should be minified. Remove all useless spaces, tabs, or newlines inside HTML tags. It should not change the original HTML tags, attributes, or values.

<!-- Input: -->

<p align="center">
<img src="https://img.shields.io/badge/Markdown-000000?style=for-the-badge&logo=markdown&logoColor=white" alt="Markdown" /></p>

<!-- Correct Output should be: -->

<p align="center"><img src="https://img.shields.io/badge/Markdown-000000?style=for-the-badge&logo=markdown&logoColor=white" alt="Markdown" /></p>

## Right-Align Example

Centering and right alignment belong to HTML and CSS rather than core Markdown. In an environment that permits raw HTML, use a paragraph with `text-align: center` or `text-align: right`. A reusable class and stylesheet are easier to maintain for repeated use.

<p style="text-align: right;">Right-aligned text</p>

Sanitizers may remove `style` attributes, and some processors do not parse Markdown inside an HTML block. Always test the destination. GFM tables have their own alignment syntax: `:---` for left, `:---:` for center, and `---:` for right.

Images can be centered with an HTML wrapper or a block class using automatic inline margins. Add `max-width: 100%` to avoid overflow. Print CSS can override screen alignment, so inspect the saved PDF.

<p style="text-align: center;"><img src="../../../logo.svg" alt="logo" style="max-width: 100%;" /></p>
