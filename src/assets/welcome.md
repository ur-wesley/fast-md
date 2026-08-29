---
title: Fast Native Markdown & MDX Viewer
description: Ultra-fast, lightweight desktop documentation viewer built with Dioxus and Rust.
author: Dioxus Fast-MD
date: 2026-08-26
tags: [rust, dioxus, markdown, mdx, desktop]
---

# Welcome to Fast-MD 🚀

**Fast-MD** is a native, high-performance Markdown and MDX reader engineered in **Rust** with **Dioxus 0.6**. It starts up instantly, parses documents natively with zero JavaScript lag, and provides live auto-reload when files are modified in your favorite editor.

---

## ⚡ Key Highlights

- **Instant Launch**: Native binary performance with Windows WebView2 integration.
- **GFM & MDX Native Rendering**: Full support for tables, task lists, footnotes, frontmatter, and JSX components.
- **Native Syntax Highlighting**: Powered by `syntect` tokenization for blazing fast code rendering.
- **Live File Watcher**: Automatically re-renders files on save via `notify`.
- **Keyboard Navigation**: `Ctrl+O` (Open), `Ctrl+F` (Search), `Ctrl+T` (New Tab), `Ctrl+W` (Close Tab), `Ctrl+,` (Settings), `Esc` / `Ctrl+Shift+F` (Zen Mode).

---

## 💻 Rust Code Snippet Example

```rust
use dioxus::prelude::*;

#[component]
pub fn Counter() -> Element {
    let mut count = use_signal(|| 0);

    rsx! {
        div {
            class: "p-4 border rounded shadow",
            button {
                onclick: move |_| count += 1,
                "Clicks: {count}"
            }
        }
    }
}
```

---

## 📦 MDX Component Showcase

<Callout type="info">
  This is a custom MDX **Callout** component rendered natively without external web dependencies.
</Callout>

<Warning>
  You can edit this file in VSCode or Neovim and watch it live-update in real time!
</Warning>

<Card>
  ### Interactive Documentation
  Organize docs with sidebars, search through headings instantly, and export to standalone HTML.
</Card>

---

## 📋 Task List & Tables

- [x] High-performance Rust parsing
- [x] Zero-panic runtime architecture
- [x] Catppuccin theme family (Mocha, Macchiato, Frappé, Latte) & Classic themes
- [x] Interactive primary color picker with palette presets
- [x] Centralized reactive state store with file-based JSON persistence

| Feature | Fast-MD (Dioxus) | Typical Electron Viewer |
| :--- | :--- | :--- |
| **Startup Time** | **< 150ms** | 1.5s - 3.5s |
| **Memory Usage** | **~35 MB** | 180MB - 350MB |
| **Code Highlighting** | **Native Rust (syntect)** | Client-side JS |
| **Live Reload** | **OS Event (notify)** | Polling / Dev server |

---

*Enjoy reading documentation at native speed!*
