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
- **Keyboard Navigation**: <kbd>Ctrl</kbd> + <kbd>O</kbd> (Open), <kbd>Ctrl</kbd> + <kbd>F</kbd> (Search), <kbd>Ctrl</kbd> + <kbd>T</kbd> (New Tab), <kbd>Ctrl</kbd> + <kbd>W</kbd> (Close Tab), <kbd>Ctrl</kbd> + <kbd>,</kbd> (Settings), <kbd>Esc</kbd> / <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>F</kbd> (Zen Mode).

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

## 📋 Modern Task Lists & Tables

- [x] High-performance Rust parsing with zero JS runtime lag
- [x] Modern typography, sleek tables, and custom styled checkboxes
- [x] Catppuccin theme family (Mocha, Macchiato, Frappé, Latte) & Classic themes
- [x] Interactive primary color picker with palette presets
- [ ] Try clicking on checkboxes directly in the viewer to toggle tasks
- [ ] Customize keyboard shortcuts and editor preferences in Settings

| Feature | Fast-MD (Dioxus) | Typical Electron Viewer |
| :--- | :--- | :--- |
| **Startup Time** | **< 150ms** | 1.5s - 3.5s |
| **Memory Usage** | **~35 MB** | 180MB - 350MB |
| **Code Highlighting** | **Native Rust (syntect)** | Client-side JS |
| **Live Reload** | **OS Event (notify)** | Polling / Dev server |

---

*Enjoy reading documentation at native speed!*
