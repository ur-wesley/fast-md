# Fast-MD 🚀

A lightning-fast, lightweight native desktop Markdown & MDX viewer built with **Dioxus 0.6** and **Rust**.

Designed for instant startup (<150ms), zero client-side JavaScript delay, and real-time live reloading.

---

## Features

- ⚡ **Instant Launch**: Native desktop app using Windows WebView2 / macOS WebKit with in-memory bundled assets.
- 📝 **Markdown & MDX**: Full CommonMark / GFM support (tables, tasklists, footnotes, strikethrough, blockquotes) plus native support for MDX JSX tags (`<Callout>`, `<Note>`, `<Warning>`, `<Card>`, `<Badge>`, `<Steps>`).
- 🏷️ **Frontmatter Extractor**: Visualized YAML/TOML metadata cards (author, date, tags, description).
- 🎨 **Native Syntax Highlighting**: Powered by `syntect` tokenization for 100+ programming languages with one-click copy code snippet actions.
- 🔄 **Live File Watcher**: Automatic real-time document reload on save via `notify`.
- 📑 **Outline & File Tree Sidebar**: Auto-generated Table of Contents with smooth anchor navigation and recursive folder browsing.
- 🗂️ **Multi-Tab Architecture**: Open multiple documents in tabs with quick switching.
- 🔍 **In-Document Search (`Ctrl+F` / `Cmd+F`)**: Instant substring search with next/previous matching.
- 🌗 **Multi-Theme System**: **GitHub Dark**, **Obsidian Night**, **GitHub Light**, **Nordic Frost**, **Solarized Dark**, and full **Catppuccin** palette.
- 🧘 **Zen Mode (`Ctrl+Shift+F`)**: Distraction-free reading view hiding all sidebars and toolbars.
- 📤 **Export to Standalone HTML**: Generate self-contained styled HTML documents.

---

## Code Quality & Standards

- **Zero-Panic Policy**: Denied `unwrap_used`, `expect_used`, and `panic` in application logic.
- **Strict Clippy Compliance**: Verified under `pedantic` and `nursery` lint suites with zero warnings.
- **Error Handling**: Contextual errors powered by `eyre`.
- **Pure Pipelines**: Data operations structured with `itertools` and iterator adapters.

---

## Keyboard Shortcuts

| Windows / Linux | macOS | Action |
| :--- | :--- | :--- |
| `Ctrl + O` | `Cmd + O` | Open File Dialog |
| `Ctrl + F` | `Cmd + F` | Toggle In-Document Search |
| `Ctrl + T` | `Cmd + T` | New Document Tab |
| `Ctrl + W` | `Cmd + W` | Close Active Tab |
| `Ctrl + +` / `Ctrl + =` | `Cmd + +` / `Cmd + =` | Zoom In |
| `Ctrl + -` | `Cmd + -` | Zoom Out |
| `Ctrl + 0` | `Cmd + 0` | Reset Zoom to 100% |
| `Ctrl + Shift + F` | `Cmd + Shift + F` | Toggle Zen Reading Mode |
| `Ctrl + ,` | `Cmd + ,` | Open Settings |

---

## Usage

### Run from Source (`cargo`)

```bash
# Run with welcome guide
cargo run

# Open a specific file
cargo run -- sample_docs/demo.mdx

# Open a documentation directory
cargo run -- docs/

# Launch directly in Zen mode with a specific theme
cargo run -- --zen --theme nord sample_docs/demo.mdx
```

### Run with Dioxus CLI (`dx`)

```bash
# Serve with hot reloading & asset tracking
dx serve

# Serve with a specific document
dx serve -- sample_docs/demo.mdx

# Build release desktop binary
dx build --release
```

### Run Tests & Verification

```bash
# Run unit tests
cargo test

# Run strict Clippy verification
cargo clippy --all-targets
```

---

## Release & Deployment

Pre-built binaries for **Windows (x86_64)** and **macOS (Apple Silicon)** are automatically built and published to GitHub Releases whenever a version tag is pushed:

```bash
git tag v0.1.0
git push origin v0.1.0
```

---

## License

This project is licensed under the [MIT License](LICENSE).
