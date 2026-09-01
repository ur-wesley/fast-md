// WYSIWYG Actions & Active Components Engine

window.syncWysiwygContent = function () {
  if (!window.serializeWysiwygToMarkdown) return;
  const md = window.serializeWysiwygToMarkdown();
  if (md !== null && md !== undefined) {
    window._lastWysiwygMd = md;
    if (window.__docSyncHandler) {
      window.__docSyncHandler({ action: "set_markdown", content: md });
    } else if (window.__wysiwygChangeHandler) {
      window.__wysiwygChangeHandler(md);
    }
  }
};

let _wysiwygDebounceTimer: ReturnType<typeof setTimeout> | null = null;
window.debouncedSyncWysiwyg = function (delayMs) {
  if (_wysiwygDebounceTimer) clearTimeout(_wysiwygDebounceTimer);
  _wysiwygDebounceTimer = setTimeout(() => {
    window.syncWysiwygContent();
  }, delayMs || 200);
};

window.formatWysiwyg = function (cmd, val) {
  const el = document.getElementById("wysiwyg-editor-surface");
  if (!el) return;
  el.focus();
  document.execCommand(cmd, false, val ?? undefined);
  if (window.updateToolbarActiveStates) window.updateToolbarActiveStates();
  window.syncWysiwygContent();
};

window.formatWysiwygHeading = function (tag) {
  const el = document.getElementById("wysiwyg-editor-surface");
  if (!el) return;
  el.focus();

  const sel = window.getSelection();
  let currentTag = "";
  if (sel && sel.rangeCount > 0) {
    let node = sel.anchorNode;
    if (node && node.nodeType === Node.TEXT_NODE) node = node.parentNode;
    while (node && node !== el) {
      const hn = node as HTMLElement;
      if (hn.tagName && /^h[1-6]$/i.test(hn.tagName)) {
        currentTag = hn.tagName.toLowerCase();
        break;
      }
      node = node.parentNode;
    }
  }

  if (currentTag === tag.toLowerCase()) {
    document.execCommand("formatBlock", false, "p");
  } else {
    document.execCommand("formatBlock", false, tag);
  }
  if (window.updateToolbarActiveStates) window.updateToolbarActiveStates();
  window.syncWysiwygContent();
};

window.formatWysiwygCode = function () {
  const sel = window.getSelection();
  if (!sel || !sel.rangeCount) return;
  const range = sel.getRangeAt(0);

  let node = sel.anchorNode;
  if (node && node.nodeType === Node.TEXT_NODE) node = node.parentNode;
  let codeNode: HTMLElement | null = null;
  while (node && (node as HTMLElement).id !== "wysiwyg-editor-surface") {
    const hn = node as HTMLElement;
    if (hn.tagName && hn.tagName.toLowerCase() === "code") {
      codeNode = hn;
      break;
    }
    node = node.parentNode;
  }

  if (codeNode) {
    const text = document.createTextNode(codeNode.textContent || "");
    codeNode.parentNode!.replaceChild(text, codeNode);
  } else {
    const code = document.createElement("code");
    code.textContent = range.toString() || "code";
    range.deleteContents();
    range.insertNode(code);
  }
  if (window.updateToolbarActiveStates) window.updateToolbarActiveStates();
  window.syncWysiwygContent();
};

window.formatWysiwygBlockquote = function () {
  const el = document.getElementById("wysiwyg-editor-surface");
  if (!el) return;
  el.focus();
  document.execCommand("formatBlock", false, "blockquote");
  window.syncWysiwygContent();
};

window.insertWysiwygCodeBlock = function () {
  const el = document.getElementById("wysiwyg-editor-surface");
  if (!el) return;
  el.focus();
  document.execCommand(
    "insertHTML",
    false,
    "<pre><code>// Code snippet\n</code></pre><p><br></p>",
  );
  window.syncWysiwygContent();
};

window.insertWysiwygTable = function () {
  const el = document.getElementById("wysiwyg-editor-surface");
  if (!el) return;
  el.focus();
  document.execCommand(
    "insertHTML",
    false,
    "<table><thead><tr><th>Header 1</th><th>Header 2</th></tr></thead><tbody><tr><td>Value 1</td><td>Value 2</td></tr></tbody></table><p><br></p>",
  );
  window.syncWysiwygContent();
};

window.insertWysiwygCallout = function (type) {
  const el = document.getElementById("wysiwyg-editor-surface");
  if (!el) return;
  el.focus();
  document.execCommand(
    "insertHTML",
    false,
    `<div class="mdx-callout mdx-callout-${type || "info"}"><p>Callout note description</p></div><p><br></p>`,
  );
  window.syncWysiwygContent();
};

window.insertWysiwygTaskList = function () {
  const el = document.getElementById("wysiwyg-editor-surface");
  if (!el) return;
  el.focus();
  document.execCommand(
    "insertHTML",
    false,
    '<ul class="task-list"><li><input type="checkbox"> Task item</li></ul><p><br></p>',
  );
  window.syncWysiwygContent();
};

window.promptWysiwygLink = function () {
  const url = prompt("Enter URL:");
  if (url) {
    document.execCommand("createLink", false, url);
    window.syncWysiwygContent();
  }
};

window.promptWysiwygImage = function () {
  const url = prompt("Enter Image URL:");
  if (url) {
    document.execCommand("insertImage", false, url);
    window.syncWysiwygContent();
  }
};

// Global Event Handler for Checkboxes & Active Components
document.addEventListener(
  "change",
  function (e) {
    const target = e.target as HTMLElement | null;
    if (!target) return;

    if (
      target.tagName &&
      target.tagName.toLowerCase() === "input" &&
      (target as HTMLInputElement).type === "checkbox"
    ) {
      const cb = target as HTMLInputElement;
      const wysiwygSurface = cb.closest("#wysiwyg-editor-surface");
      if (wysiwygSurface) {
        if (cb.checked) {
          cb.setAttribute("checked", "checked");
        } else {
          cb.removeAttribute("checked");
        }
        window.syncWysiwygContent();
        return;
      }

      const previewContainer = target.closest(".markdown-body");
      if (previewContainer) {
        const allCheckboxes = Array.from(
          previewContainer.querySelectorAll('input[type="checkbox"]'),
        );
        const idx = allCheckboxes.indexOf(target as HTMLInputElement);
        if (idx !== -1 && window.__docSyncHandler) {
          window.__docSyncHandler({
            action: "toggle_checkbox",
            index: idx,
            checked: (target as HTMLInputElement).checked,
          });
        }
        return;
      }
    }
  },
  true,
);

document.addEventListener(
  "click",
  function (e) {
    const target = e.target as HTMLElement | null;
    if (!target) return;

    if (
      target.tagName &&
      target.tagName.toLowerCase() === "input" &&
      (target as HTMLInputElement).type === "checkbox"
    ) {
      const wysiwygSurface = target.closest("#wysiwyg-editor-surface");
      if (wysiwygSurface) {
        setTimeout(() => {
          const cb = target as HTMLInputElement;
          if (cb.checked) {
            cb.setAttribute("checked", "checked");
          } else {
            cb.removeAttribute("checked");
          }
          window.syncWysiwygContent();
        }, 0);
        return;
      }

      const previewContainer = target.closest(".markdown-body");
      if (previewContainer) {
        setTimeout(() => {
          const cb = target as HTMLInputElement;
          const allCheckboxes = Array.from(
            previewContainer.querySelectorAll('input[type="checkbox"]'),
          );
          const idx = allCheckboxes.indexOf(cb);
          if (idx !== -1 && window.__docSyncHandler) {
            window.__docSyncHandler({
              action: "toggle_checkbox",
              index: idx,
              checked: cb.checked,
            });
          }
        }, 0);
        return;
      }
    }

    if (target.tagName && target.tagName.toLowerCase() === "summary") {
      const details = target.closest("details");
      const wysiwygSurface = target.closest("#wysiwyg-editor-surface");
      if (details && wysiwygSurface) {
        setTimeout(() => {
          window.syncWysiwygContent();
        }, 50);
      }
    }
  },
  true,
);

// Input listener on WYSIWYG surface for instant real-time sync
document.addEventListener("input", function (e) {
  if (
    e.target &&
    ((e.target as HTMLElement).id === "wysiwyg-editor-surface" ||
      (e.target as HTMLElement).closest("#wysiwyg-editor-surface"))
  ) {
    window.syncWysiwygContent();
  }
});

window.flushWysiwygContent = function () {
  if (_wysiwygDebounceTimer) {
    clearTimeout(_wysiwygDebounceTimer);
    _wysiwygDebounceTimer = null;
  }
  window.syncWysiwygContent();
};

window.prepareDocumentModeChange = function () {
  window.flushWysiwygContent();
};

function wysiwygAncestor(node, surface, matcher) {
  while (node && node !== surface) {
    if (node.nodeType === Node.ELEMENT_NODE && matcher(node)) return node;
    node = node.parentNode;
  }
  return null;
}

function wysiwygIsInCodeBlock(node, surface) {
  return !!wysiwygAncestor(node, surface, (el) => {
    const tag = el.tagName ? el.tagName.toLowerCase() : "";
    return (
      tag === "pre" ||
      tag === "code" ||
      (el.classList && el.classList.contains("code-block-container"))
    );
  });
}

function wysiwygLiText(li) {
  let text = "";
  for (const child of li.childNodes) {
    if (
      child.nodeType === Node.ELEMENT_NODE &&
      child.tagName.toLowerCase() === "input" &&
      child.type === "checkbox"
    ) {
      continue;
    }
    text += child.textContent || "";
  }
  return text.trim();
}

function wysiwygPlaceCaretIn(node, offset) {
  const range = document.createRange();
  const sel = window.getSelection();
  if (!sel) return;
  if (node.nodeType === Node.TEXT_NODE) {
    range.setStart(node, Math.min(offset, node.textContent.length));
    range.collapse(true);
  } else {
    range.selectNodeContents(node);
    range.collapse(true);
  }
  sel.removeAllRanges();
  sel.addRange(range);
}

function wysiwygPlaceCaretInCell(cell) {
  let target = cell;
  if (!target.childNodes.length) {
    target.appendChild(document.createElement("br"));
  }
  const first = target.firstChild;
  if (first && first.nodeType === Node.TEXT_NODE) {
    wysiwygPlaceCaretIn(first, 0);
  } else {
    wysiwygPlaceCaretIn(target, 0);
  }
}

function wysiwygEnsureTaskCheckbox(li) {
  if (!li || li.querySelector('input[type="checkbox"]')) return;
  const cb = document.createElement("input");
  cb.type = "checkbox";
  li.insertBefore(cb, li.firstChild);
  if (
    cb.nextSibling &&
    cb.nextSibling.nodeType === Node.TEXT_NODE &&
    cb.nextSibling.textContent &&
    !cb.nextSibling.textContent.startsWith(" ")
  ) {
    cb.nextSibling.textContent = " " + cb.nextSibling.textContent;
  } else {
    li.insertBefore(document.createTextNode(" "), cb.nextSibling);
  }
}

function wysiwygHandleTableEnter(surface, cell) {
  const tr = cell.closest("tr");
  const table = cell.closest("table");
  if (!tr || !table) return false;

  const colCount = tr.cells.length;
  if (colCount === 0) return false;

  const rowEmpty = Array.from(tr.cells).every(
    (c) => (c as HTMLTableCellElement).textContent!.trim() === "",
  );
  const tbody = table.tBodies[0] || table;
  const bodyRows = tbody.rows
    ? Array.from(tbody.rows)
    : Array.from(table.querySelectorAll("tr"));
  const isLastRow = bodyRows.length > 0 && bodyRows[bodyRows.length - 1] === tr;

  if (rowEmpty && isLastRow) {
    tr.remove();
    const p = document.createElement("p");
    p.appendChild(document.createElement("br"));
    table.parentNode.insertBefore(p, table.nextSibling);
    wysiwygPlaceCaretIn(p, 0);
    return true;
  }

  const newTr = document.createElement("tr");
  for (let i = 0; i < colCount; i++) {
    const td = document.createElement("td");
    td.appendChild(document.createElement("br"));
    newTr.appendChild(td);
  }

  const parent = tr.parentNode;
  if (tr.parentNode && tr.parentNode.tagName.toLowerCase() === "thead") {
    let body = table.tBodies[0];
    if (!body) {
      body = document.createElement("tbody");
      table.appendChild(body);
    }
    body.appendChild(newTr);
  } else {
    parent.insertBefore(newTr, tr.nextSibling);
  }

  wysiwygPlaceCaretInCell(newTr.cells[0]);
  return true;
}

function revealWysiwygCaret() {
  requestAnimationFrame(function () {
    const surface = document.getElementById("wysiwyg-editor-surface");
    const scrollArea = document.getElementById("wysiwyg-scroll-area");
    if (!surface || !scrollArea) return;

    const sel = window.getSelection();
    if (!sel || !sel.rangeCount) return;
    if (!surface.contains(sel.anchorNode)) return;

    let node = sel.anchorNode;
    if (node && node.nodeType === Node.TEXT_NODE) node = node.parentNode;

    const pre = wysiwygAncestor(node, surface, (el) => {
      const tag = el.tagName ? el.tagName.toLowerCase() : "";
      return tag === "pre";
    });
    if (pre) {
      pre.scrollLeft = 0;
    }

    const range = sel.getRangeAt(0);
    let rect = range.getBoundingClientRect();
    if (rect.height === 0) {
      const rects = range.getClientRects();
      if (rects.length > 0) rect = rects[0];
    }

    const areaRect = scrollArea.getBoundingClientRect();
    const padding = 8;

    if (rect.bottom > areaRect.bottom - padding) {
      scrollArea.scrollTop += rect.bottom - areaRect.bottom + padding;
    } else if (rect.top < areaRect.top + padding) {
      scrollArea.scrollTop -= areaRect.top + padding - rect.top;
    }
  });
}

document.addEventListener(
  "keydown",
  function (e) {
    const surface = document.getElementById("wysiwyg-editor-surface");
    const inWysiwyg = surface && surface.contains(document.activeElement);

    if (
      e.key === "Tab" &&
      inWysiwyg &&
      !e.ctrlKey &&
      !e.metaKey &&
      !e.altKey &&
      !e.isComposing
    ) {
      const sel = window.getSelection();
      let node = sel && sel.anchorNode;
      if (node && node.nodeType === Node.TEXT_NODE) node = node.parentNode;
      if (wysiwygIsInCodeBlock(node, surface)) return;
      e.preventDefault();
      if (e.shiftKey) {
        document.execCommand("outdent");
      } else {
        const sel = window.getSelection();
        let node = sel && sel.anchorNode;
        if (node && node.nodeType === Node.TEXT_NODE) node = node.parentNode;
        const li =
          node &&
          wysiwygAncestor(
            node,
            surface,
            (el) => el.tagName && el.tagName.toLowerCase() === "li",
          );
        if (li) {
          document.execCommand("indent");
        } else {
          document.execCommand("insertText", false, "  ");
        }
      }
      return;
    }

    if (
      e.key !== "Enter" ||
      e.shiftKey ||
      e.ctrlKey ||
      e.metaKey ||
      e.altKey ||
      e.isComposing
    )
      return;

    if (!inWysiwyg) return;

    const sel = window.getSelection();
    if (!sel || !sel.rangeCount) return;

    let node = sel.anchorNode;
    if (node && node.nodeType === Node.TEXT_NODE) node = node.parentNode;
    if (wysiwygIsInCodeBlock(node, surface)) return;

    const cell = wysiwygAncestor(node, surface, (el) => {
      const tag = el.tagName ? el.tagName.toLowerCase() : "";
      return tag === "td" || tag === "th";
    });
    if (cell) {
      e.preventDefault();
      if (wysiwygHandleTableEnter(surface, cell)) {
        window.syncWysiwygContent();
        revealWysiwygCaret();
      }
      return;
    }

    const li = wysiwygAncestor(
      node,
      surface,
      (el) => el.tagName && el.tagName.toLowerCase() === "li",
    );
    const taskList = li ? li.closest("ul.task-list") : null;
    if (taskList && li) {
      if (wysiwygLiText(li) === "") {
        e.preventDefault();
        document.execCommand("outdent");
        if (wysiwygLiText(li) === "" && li.parentNode) {
          li.remove();
        }
        window.syncWysiwygContent();
        revealWysiwygCaret();
        return;
      }

      requestAnimationFrame(() => {
        const activeLi = wysiwygAncestor(
          sel.anchorNode,
          surface,
          (el) => el.tagName && el.tagName.toLowerCase() === "li",
        );
        if (activeLi && activeLi.closest("ul.task-list")) {
          wysiwygEnsureTaskCheckbox(activeLi);
          window.syncWysiwygContent();
        }
      });
    }
  },
  true,
);

document.addEventListener(
  "keyup",
  function (e) {
    const surface = document.getElementById("wysiwyg-editor-surface");
    const inWysiwyg = surface && surface.contains(document.activeElement);
    if (!inWysiwyg) return;
    if (
      e.key !== "Enter" ||
      e.ctrlKey ||
      e.metaKey ||
      e.altKey ||
      e.isComposing
    )
      return;
    revealWysiwygCaret();
  },
  true,
);

// HTML-to-Markdown Serializer for WYSIWYG
window.serializeWysiwygToMarkdown = function () {
  const surface = document.getElementById("wysiwyg-editor-surface");
  if (!surface) return null;

  const configContainer = surface.querySelector(".config-doc-container");
  if (configContainer) {
    const codeContent =
      configContainer.querySelector(".code-content pre") ||
      configContainer.querySelector("pre");
    if (codeContent) {
      return codeContent.textContent.replace(/\n+$/, "") + "\n";
    }
    return null;
  }

  const plainTextDoc = surface.querySelector(".plain-text-doc");
  if (plainTextDoc) {
    return plainTextDoc.textContent.replace(/\n+$/, "") + "\n";
  }

  function nodeToMd(node: Node): string {
    if (node.nodeType === Node.TEXT_NODE) {
      return node.textContent || "";
    }
    if (node.nodeType !== Node.ELEMENT_NODE) return "";

    const el = node as HTMLElement;
    const tag = el.tagName.toLowerCase();
    let inner = Array.from(el.childNodes).map(nodeToMd).join("");

    switch (tag) {
      case "h1":
        return "# " + inner.trim() + "\n\n";
      case "h2":
        return "## " + inner.trim() + "\n\n";
      case "h3":
        return "### " + inner.trim() + "\n\n";
      case "h4":
        return "#### " + inner.trim() + "\n\n";
      case "h5":
        return "##### " + inner.trim() + "\n\n";
      case "h6":
        return "###### " + inner.trim() + "\n\n";
      case "p":
        return inner.trim() ? inner.trim() + "\n\n" : "\n";
      case "strong":
      case "b":
        return "**" + inner + "**";
      case "em":
      case "i":
        return "*" + inner + "*";
      case "del":
      case "s":
      case "strike":
        return "~~" + inner + "~~";
      case "kbd":
        return "<kbd>" + inner + "</kbd>";
      case "code":
        if (
          el.parentNode &&
          (el.parentNode as HTMLElement).tagName.toLowerCase() === "pre"
        ) {
          return inner;
        }
        return "`" + inner + "`";
      case "pre": {
        const codeNode = el.querySelector("code");
        const codeText = codeNode ? codeNode.textContent : el.textContent;
        if (el.closest(".config-doc-container")) {
          return (codeText || "").replace(/\n+$/, "");
        }
        const lang = el.getAttribute("data-lang") || "";
        return (
          "```" +
          lang +
          "\n" +
          (codeText || "").replace(/\n+$/, "") +
          "\n```\n\n"
        );
      }
      case "blockquote":
        return (
          inner
            .split("\n")
            .map((l: string) => (l ? "> " + l : ">"))
            .join("\n") + "\n\n"
        );
      case "ul":
        return (
          Array.from(el.children)
            .map((li) => {
              const liEl = li as HTMLElement;
              const chk = liEl.querySelector(
                "input[type=checkbox]",
              ) as HTMLInputElement | null;
              if (chk) {
                const isChecked = chk.checked || chk.hasAttribute("checked");
                let text = nodeToMd(li)
                  .replace(/^\[[ xX]\]\s*/, "")
                  .trim();
                return "- [" + (isChecked ? "x" : " ") + "] " + text;
              }
              return "- " + nodeToMd(li).trim();
            })
            .join("\n") + "\n\n"
        );
      case "ol":
        return (
          Array.from(el.children)
            .map((li, idx) => `${idx + 1}. ` + nodeToMd(li).trim())
            .join("\n") + "\n\n"
        );
      case "li": {
        let liText = "";
        for (const child of el.childNodes) {
          if (
            child.nodeType === Node.ELEMENT_NODE &&
            (child as HTMLElement).tagName.toLowerCase() === "input" &&
            (child as HTMLInputElement).type === "checkbox"
          ) {
            continue;
          }
          liText += nodeToMd(child);
        }
        return liText;
      }
      case "hr":
        return "---\n\n";
      case "a": {
        const href = el.getAttribute("href") || "";
        if (el.classList && el.classList.contains("heading-anchor")) return "";
        return "[" + inner + "](" + href + ")";
      }
      case "img":
        return (
          "![" +
          (el.getAttribute("alt") || "") +
          "](" +
          (el.getAttribute("src") || "") +
          ")"
        );
      case "table": {
        const rows = Array.from(
          el.querySelectorAll("tr"),
        ) as HTMLTableRowElement[];
        if (rows.length === 0) return "";
        let mdTable = "";
        rows.forEach((row, rIdx) => {
          const cells = Array.from(row.querySelectorAll("th, td")).map((c) =>
            nodeToMd(c).trim(),
          );
          mdTable += "| " + cells.join(" | ") + " |\n";
          if (rIdx === 0) {
            mdTable += "| " + cells.map(() => "---").join(" | ") + " |\n";
          }
        });
        return mdTable + "\n";
      }
      case "details": {
        const summary = el.querySelector("summary");
        const summaryText = summary ? nodeToMd(summary).trim() : "Details";
        const clone = el.cloneNode(true) as HTMLElement;
        const cloneSummary = clone.querySelector("summary");
        if (cloneSummary) cloneSummary.remove();
        const detailsContent = Array.from(clone.childNodes)
          .map(nodeToMd)
          .join("")
          .trim();
        const isOpen = el.hasAttribute("open") ? " open" : "";
        return `<details${isOpen}>\n<summary>${summaryText}</summary>\n\n${detailsContent}\n</details>\n\n`;
      }
      case "summary":
        return inner;
      case "div": {
        if (el.classList.contains("code-block-container")) {
          const codeContent =
            el.querySelector(".code-content pre") || el.querySelector("pre");
          const text = codeContent ? codeContent.textContent : inner;
          if (
            el.classList.contains("config-code-block") ||
            el.closest(".config-doc-container")
          ) {
            return (text || "").replace(/\n+$/, "") + "\n";
          }
          const langLabel = el.querySelector(".code-lang-label");
          const lang = langLabel
            ? (langLabel.textContent || "").trim().toLowerCase()
            : "";
          return (
            "```" +
            (lang === "text" ? "" : lang) +
            "\n" +
            (text || "").replace(/\n+$/, "") +
            "\n```\n\n"
          );
        }
        if (el.classList.contains("config-doc-container")) {
          const codeContent =
            el.querySelector(".code-content pre") || el.querySelector("pre");
          const text = codeContent ? codeContent.textContent : "";
          return (text || "").replace(/\n+$/, "") + "\n";
        }
        if (el.classList.contains("mdx-callout")) {
          let type = "info";
          if (el.classList.contains("mdx-callout-warning")) type = "warning";
          else if (el.classList.contains("mdx-callout-danger")) type = "danger";
          else if (el.classList.contains("mdx-callout-tip")) type = "tip";
          else if (el.classList.contains("mdx-callout-note")) type = "note";
          return `<Callout type="${type}">\n${inner.trim()}\n</Callout>\n\n`;
        }
        if (el.classList.contains("mdx-card")) {
          return `<Card>\n${inner.trim()}\n</Card>\n\n`;
        }
        if (el.classList.contains("mdx-steps")) {
          return `<Steps>\n${inner.trim()}\n</Steps>\n\n`;
        }
        return inner ? inner + "\n" : "";
      }
      case "span": {
        if (el.classList.contains("mdx-badge")) {
          return `<Badge>${inner.trim()}</Badge>`;
        }
        return inner;
      }
      case "section": {
        if (el.classList.contains("markdown-section")) {
          return inner.trim() ? inner.trim() + "\n\n" : "";
        }
        return inner;
      }
      case "input":
        return "";
      case "button":
        return "";
      case "br":
        return "\n";
      default:
        return inner;
    }
  }

  return Array.from(surface.childNodes).map(nodeToMd).join("").trim();
};
