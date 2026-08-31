
window.copyCodeSnippet = function(btn) {
    try {
        const code = btn.getAttribute('data-code');
        if (code) {
            navigator.clipboard.writeText(code).then(() => {
                const span = btn.querySelector('span');
                if (span) {
                    const orig = span.innerText;
                    span.innerText = 'Copied!';
                    setTimeout(() => { span.innerText = orig; }, 1800);
                }
            }).catch(err => console.error('Copy failed:', err));
        }
    } catch(e) { console.error(e); }
};

function slugifyHeading(text) {
    let slug = '';
    let prevDash = false;
    for (const c of text) {
        const isAlnum = /[0-9A-Za-z]/.test(c) || c.toLowerCase() !== c.toUpperCase();
        if (isAlnum) {
            slug += c.toLowerCase();
            prevDash = false;
        } else if ((c === ' ' || c === '-' || c === '_') && !prevDash && slug) {
            slug += '-';
            prevDash = true;
        }
    }
    if (slug.endsWith('-')) slug = slug.slice(0, -1);
    return slug || 'section';
}

window.slugifyHeading = slugifyHeading;

function nearestScroller(el) {
    const named = [
        document.getElementById('viewer-scroll-area'),
        document.getElementById('split-preview-scroll-area'),
        document.getElementById('wysiwyg-scroll-area'),
    ];
    for (const c of named) {
        if (c && c.contains(el)) return c;
    }
    let n = el.parentElement;
    while (n && n !== document.body) {
        const oy = getComputedStyle(n).overflowY;
        if (oy === 'auto' || oy === 'scroll') return n;
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
    const ta = document.getElementById('source-markdown-textarea');
    if (!ta) return false;
    const lines = ta.value.split('\n');
    let pos = 0;
    for (let i = 0; i < lines.length; i++) {
        const m = lines[i].match(/^(#{1,6})\s+(.+)$/);
        if (m && slugifyHeading(m[2].trim()) === id) {
            const lineHeight = parseFloat(getComputedStyle(ta).lineHeight) || 21;
            ta.scrollTop = i * lineHeight;
            ta.focus();
            ta.setSelectionRange(pos, pos + lines[i].length);
            if (window.onEditorSourceScroll) window.onEditorSourceScroll();
            return true;
        }
        pos += lines[i].length + 1;
    }
    return false;
}

window.scrollToSection = function(id) {
    try {
        if (!id) return;
        const el = document.getElementById(id);
        if (el) {
            const scroller = nearestScroller(el);
            if (scroller) {
                scrollContainerToEl(scroller, el);
            } else {
                el.scrollIntoView({ behavior: 'smooth', block: 'start' });
            }
        }
        scrollTextareaToHeading(id);
        if (window.saveCurrentScroll) window.saveCurrentScroll();
    } catch(e) { console.error(e); }
};

