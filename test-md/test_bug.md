# 🥦 Linked-Mind
- Usage:
  ```bash
  # Generates map.toon (default TOON format)
  map-builder

  # Generates map.json (optional JSON format)
  map-builder --json
  ```
## 🧠 Why Graph-based KB for LLMs?
Standard RAG (Retrieval-Augmented Generation) often treats files as isolated chunks. However, human knowledge is a web. By using Linked-Mind, you provide the LLM with:
1. Contextual Proximity: If Node A links to Node B, the LLM knows they are related even if they don't share keywords.
2. Structural Understanding: The AI sees the hierarchy and tags, allowing it to "browse" your brain more effectively.
## 📂 Project Structure
- `src/parser.zig`: Unified parser for multiple formats (.md, .org, .txt, .pdf) extracting `[[links]]` and `#tags`.
- `src/graph.zig`: Adjacency-list based graph representation and link resolver.
- `src/li.zig`: Workspace-aware CLI with `init`, `scan`, `export`, `path`, `clusters`, `gc`, `similar`, `suggest`, `visualize`.
- `src/cache.zig`: Incremental scanning engine with `mtime` + SHA-256 cache.
- `src/main.zig`: Legacy CLI handler (direct path mode).

Built with speed and precision in Zig.
