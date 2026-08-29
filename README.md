# Fast-MD

A lightweight, native desktop Markdown and MDX viewer built with Dioxus 0.6 and Rust.

## Features

- **Markdown and MDX**: Full CommonMark/GFM support and native MDX components (`<Callout>`, `<Note>`, `<Warning>`, `<Card>`, `<Badge>`, `<Steps>`).
- **Frontmatter**: YAML/TOML metadata visualization.
- **Syntax Highlighting**: Tokenization for 100+ languages with copy support.
- **Live Reload**: Automatic document reload on file save.
- **Navigation**: Document outline (Table of Contents) and file tree sidebar.
- **Multi-Tab and Search**: Tabbed browsing and in-document text search (`Ctrl+F` / `Cmd+F`).
- **Themes and Zen Mode**: Dark/light themes (GitHub, Obsidian, Nord, Solarized, Catppuccin) and distraction-free Zen mode (`Ctrl+Shift+F`).
- **Export**: Export to standalone styled HTML.

## Keyboard Shortcuts

| Shortcut (Win/Linux)    | Shortcut (macOS)      | Action             |
| :---------------------- | :-------------------- | :----------------- |
| `Ctrl + O`              | `Cmd + O`             | Open file          |
| `Ctrl + F`              | `Cmd + F`             | Search in document |
| `Ctrl + T`              | `Cmd + T`             | New tab            |
| `Ctrl + W`              | `Cmd + W`             | Close active tab   |
| `Ctrl + +` / `Ctrl + =` | `Cmd + +` / `Cmd + =` | Zoom in            |
| `Ctrl + -`              | `Cmd + -`             | Zoom out           |
| `Ctrl + 0`              | `Cmd + 0`             | Reset zoom         |
| `Ctrl + Shift + F`      | `Cmd + Shift + F`     | Toggle Zen mode    |
| `Ctrl + ,`              | `Cmd + ,`             | Settings           |

## Usage

```bash
# Run with welcome guide
cargo run

# Open a specific file or directory
cargo run -- path/to/doc.md
cargo run -- path/to/folder/

# Build release binary
cargo build --release
```

## License

[MIT](LICENSE)
