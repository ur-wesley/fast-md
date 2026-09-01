window.copyTabContent = function (tabId) {
  try {
    const wrapper = document.querySelector('[data-tab-id="' + tabId + '"]');
    if (!wrapper) return;
    const ta = (wrapper.querySelector(
      'textarea[data-tab-id="' + tabId + '"]',
    ) ||
      wrapper.querySelector(
        "textarea#source-markdown-textarea",
      )) as HTMLTextAreaElement | null;
    if (!ta) return;
    navigator.clipboard
      .writeText(ta.value)
      .then(() => {
        const btn = wrapper.querySelector(
          '.copy-code-button[data-tab-id="' + tabId + '"]',
        );
        if (!btn) return;
        const span = btn.querySelector("span");
        if (span) {
          const orig = span.innerText;
          span.innerText = "Copied!";
          setTimeout(() => {
            span.innerText = orig;
          }, 1800);
        }
      })
      .catch((err) => console.error("Copy failed:", err));
  } catch (e) {
    console.error(e);
  }
};

window.copyCodeSnippet = function (btn) {
  try {
    const tabId = btn.getAttribute("data-tab-id");
    if (tabId) {
      window.copyTabContent(tabId);
      return;
    }
    const code = btn.getAttribute("data-code");
    if (code) {
      navigator.clipboard
        .writeText(code)
        .then(() => {
          const span = btn.querySelector("span");
          if (span) {
            const orig = span.innerText;
            span.innerText = "Copied!";
            setTimeout(() => {
              span.innerText = orig;
            }, 1800);
          }
        })
        .catch((err) => console.error("Copy failed:", err));
    }
  } catch (e) {
    console.error(e);
  }
};

function slugifyHeading(text) {
  let slug = "";
  let prevDash = false;
  for (const c of text) {
    const isAlnum =
      /[0-9A-Za-z]/.test(c) || c.toLowerCase() !== c.toUpperCase();
    if (isAlnum) {
      slug += c.toLowerCase();
      prevDash = false;
    } else if ((c === " " || c === "-" || c === "_") && !prevDash && slug) {
      slug += "-";
      prevDash = true;
    }
  }
  if (slug.endsWith("-")) slug = slug.slice(0, -1);
  return slug || "section";
}

window.slugifyHeading = slugifyHeading;

function nearestScroller(el) {
  const named = [
    document.getElementById("viewer-scroll-area"),
    document.getElementById("split-preview-scroll-area"),
    document.getElementById("wysiwyg-scroll-area"),
  ];
  for (const c of named) {
    if (c && c.contains(el)) return c;
  }
  let n = el.parentElement;
  while (n && n !== document.body) {
    const oy = getComputedStyle(n).overflowY;
    if (oy === "auto" || oy === "scroll") return n;
    n = n.parentElement;
  }
  return null;
}

function scrollContainerToEl(container, el) {
  const cRect = container.getBoundingClientRect();
  const eRect = el.getBoundingClientRect();
  const zoom = parseFloat(container.style.zoom) || 1;
  container.scrollTop += (eRect.top - cRect.top) / zoom;
}

function scrollTextareaToHeading(id) {
  const ta = document.getElementById(
    "source-markdown-textarea",
  ) as HTMLTextAreaElement | null;
  if (!ta) return false;
  const lines = ta.value.split("\n");
  for (let i = 0; i < lines.length; i++) {
    const m = lines[i].match(/^(#{1,6})\s+(.+)$/);
    if (m && slugifyHeading(m[2].trim()) === id) {
      const lineHeight = parseFloat(getComputedStyle(ta).lineHeight) || 21;
      ta.scrollTop = i * lineHeight;
      if (window.onEditorSourceScroll) window.onEditorSourceScroll();
      return true;
    }
  }
  return false;
}

function scrollTextareaToLine(line) {
  const ta = document.getElementById(
    "source-markdown-textarea",
  ) as HTMLTextAreaElement | null;
  if (!ta || line < 0) return false;
  const lines = ta.value.split("\n");
  if (line >= lines.length) return false;
  const lineHeight = parseFloat(getComputedStyle(ta).lineHeight) || 21;
  ta.scrollTop = line * lineHeight;
  if (window.onEditorSourceScroll) window.onEditorSourceScroll();
  return true;
}

function scrollPreToLine(line) {
  const areas = [
    document.getElementById("viewer-scroll-area"),
    document.getElementById("split-preview-scroll-area"),
    document.getElementById("wysiwyg-scroll-area"),
  ];
  for (const area of areas) {
    if (!area) continue;
    const pre = area.querySelector(
      ".config-code-block pre, .config-code-block .code-content pre, pre",
    );
    if (!pre) continue;
    const lineHeight = parseFloat(getComputedStyle(pre).lineHeight) || 21;
    const preRect = pre.getBoundingClientRect();
    const areaRect = area.getBoundingClientRect();
    const offsetInArea = preRect.top - areaRect.top + area.scrollTop;
    area.scrollTop = offsetInArea + line * lineHeight;
    return true;
  }
  return false;
}

window.scrollToSection = function (id, line) {
  try {
    if (!id) return;
    if (typeof line === "number" && line >= 0) {
      scrollTextareaToLine(line);
      scrollPreToLine(line);
      if (window.saveCurrentScroll) window.saveCurrentScroll();
      if (window.onViewerScroll) window.onViewerScroll();
      return;
    }
    const el = document.getElementById(id);
    if (el) {
      const scroller = nearestScroller(el);
      if (scroller) {
        scrollContainerToEl(scroller, el);
      } else {
        el.scrollIntoView({ block: "start" });
      }
      if (window.saveCurrentScroll) window.saveCurrentScroll();
      if (window.onViewerScroll) window.onViewerScroll();
      return;
    }
    scrollTextareaToHeading(id);
    if (window.saveCurrentScroll) window.saveCurrentScroll();
    if (window.onViewerScroll) window.onViewerScroll();
  } catch (e) {
    console.error(e);
  }
};
