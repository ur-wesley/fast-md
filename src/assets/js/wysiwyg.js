// WYSIWYG Actions & Active Components Engine

window.syncWysiwygContent = function() {
    if (!window.serializeWysiwygToMarkdown) return;
    const md = window.serializeWysiwygToMarkdown();
    if (md !== null && md !== undefined) {
        window._lastWysiwygMd = md;
        if (window.__docSyncHandler) {
            window.__docSyncHandler({ action: 'set_markdown', content: md });
        } else if (window.__wysiwygChangeHandler) {
            window.__wysiwygChangeHandler(md);
        }
    }
};

let _wysiwygDebounceTimer = null;
window.debouncedSyncWysiwyg = function(delayMs) {
    if (_wysiwygDebounceTimer) clearTimeout(_wysiwygDebounceTimer);
    _wysiwygDebounceTimer = setTimeout(() => {
        window.syncWysiwygContent();
    }, delayMs || 200);
};

window.formatWysiwyg = function(cmd, val) {
    const el = document.getElementById('wysiwyg-editor-surface');
    if (!el) return;
    el.focus();
    document.execCommand(cmd, false, val || null);
    if (window.updateToolbarActiveStates) window.updateToolbarActiveStates();
    window.syncWysiwygContent();
};

window.formatWysiwygHeading = function(tag) {
    const el = document.getElementById('wysiwyg-editor-surface');
    if (!el) return;
    el.focus();

    const sel = window.getSelection();
    let currentTag = '';
    if (sel && sel.rangeCount > 0) {
        let node = sel.anchorNode;
        if (node && node.nodeType === Node.TEXT_NODE) node = node.parentNode;
        while (node && node !== el) {
            if (node.tagName && /^h[1-6]$/i.test(node.tagName)) {
                currentTag = node.tagName.toLowerCase();
                break;
            }
            node = node.parentNode;
        }
    }

    if (currentTag === tag.toLowerCase()) {
        document.execCommand('formatBlock', false, 'p');
    } else {
        document.execCommand('formatBlock', false, tag);
    }
    if (window.updateToolbarActiveStates) window.updateToolbarActiveStates();
    window.syncWysiwygContent();
};

window.formatWysiwygCode = function() {
    const sel = window.getSelection();
    if (!sel.rangeCount) return;
    const range = sel.getRangeAt(0);

    let node = sel.anchorNode;
    if (node && node.nodeType === Node.TEXT_NODE) node = node.parentNode;
    let codeNode = null;
    while (node && node.id !== 'wysiwyg-editor-surface') {
        if (node.tagName && node.tagName.toLowerCase() === 'code') {
            codeNode = node;
            break;
        }
        node = node.parentNode;
    }

    if (codeNode) {
        const text = document.createTextNode(codeNode.textContent);
        codeNode.parentNode.replaceChild(text, codeNode);
    } else {
        const code = document.createElement('code');
        code.textContent = range.toString() || 'code';
        range.deleteContents();
        range.insertNode(code);
    }
    if (window.updateToolbarActiveStates) window.updateToolbarActiveStates();
    window.syncWysiwygContent();
};

window.formatWysiwygBlockquote = function() {
    const el = document.getElementById('wysiwyg-editor-surface');
    if (!el) return;
    el.focus();
    document.execCommand('formatBlock', false, 'blockquote');
    window.syncWysiwygContent();
};

window.insertWysiwygCodeBlock = function() {
    const el = document.getElementById('wysiwyg-editor-surface');
    if (!el) return;
    el.focus();
    document.execCommand('insertHTML', false, '<pre><code>// Code snippet\n</code></pre><p><br></p>');
    window.syncWysiwygContent();
};

window.insertWysiwygTable = function() {
    const el = document.getElementById('wysiwyg-editor-surface');
    if (!el) return;
    el.focus();
    document.execCommand('insertHTML', false, '<table><thead><tr><th>Header 1</th><th>Header 2</th></tr></thead><tbody><tr><td>Value 1</td><td>Value 2</td></tr></tbody></table><p><br></p>');
    window.syncWysiwygContent();
};

window.insertWysiwygCallout = function(type) {
    const el = document.getElementById('wysiwyg-editor-surface');
    if (!el) return;
    el.focus();
    document.execCommand('insertHTML', false, `<div class="mdx-callout mdx-callout-${type || 'info'}"><p>Callout note description</p></div><p><br></p>`);
    window.syncWysiwygContent();
};

window.insertWysiwygTaskList = function() {
    const el = document.getElementById('wysiwyg-editor-surface');
    if (!el) return;
    el.focus();
    document.execCommand('insertHTML', false, '<ul class="task-list"><li><input type="checkbox"> Task item</li></ul><p><br></p>');
    window.syncWysiwygContent();
};

window.promptWysiwygLink = function() {
    const url = prompt('Enter URL:');
    if (url) {
        document.execCommand('createLink', false, url);
        window.syncWysiwygContent();
    }
};

window.promptWysiwygImage = function() {
    const url = prompt('Enter Image URL:');
    if (url) {
        document.execCommand('insertImage', false, url);
        window.syncWysiwygContent();
    }
};

// Global Event Handler for Checkboxes & Active Components
document.addEventListener('change', function(e) {
    const target = e.target;
    if (!target) return;

    if (target.tagName && target.tagName.toLowerCase() === 'input' && target.type === 'checkbox') {
        const wysiwygSurface = target.closest('#wysiwyg-editor-surface');
        if (wysiwygSurface) {
            if (target.checked) {
                target.setAttribute('checked', 'checked');
            } else {
                target.removeAttribute('checked');
            }
            window.syncWysiwygContent();
            return;
        }

        const previewContainer = target.closest('.markdown-body');
        if (previewContainer) {
            const allCheckboxes = Array.from(previewContainer.querySelectorAll('input[type="checkbox"]'));
            const idx = allCheckboxes.indexOf(target);
            if (idx !== -1 && window.__docSyncHandler) {
                window.__docSyncHandler({ action: 'toggle_checkbox', index: idx, checked: target.checked });
            }
            return;
        }
    }
}, true);

document.addEventListener('click', function(e) {
    const target = e.target;
    if (!target) return;

    if (target.tagName && target.tagName.toLowerCase() === 'input' && target.type === 'checkbox') {
        const wysiwygSurface = target.closest('#wysiwyg-editor-surface');
        if (wysiwygSurface) {
            setTimeout(() => {
                if (target.checked) {
                    target.setAttribute('checked', 'checked');
                } else {
                    target.removeAttribute('checked');
                }
                window.syncWysiwygContent();
            }, 0);
            return;
        }

        const previewContainer = target.closest('.markdown-body');
        if (previewContainer) {
            setTimeout(() => {
                const allCheckboxes = Array.from(previewContainer.querySelectorAll('input[type="checkbox"]'));
                const idx = allCheckboxes.indexOf(target);
                if (idx !== -1 && window.__docSyncHandler) {
                    window.__docSyncHandler({ action: 'toggle_checkbox', index: idx, checked: target.checked });
                }
            }, 0);
            return;
        }
    }

    if (target.tagName && target.tagName.toLowerCase() === 'summary') {
        const details = target.closest('details');
        const wysiwygSurface = target.closest('#wysiwyg-editor-surface');
        if (details && wysiwygSurface) {
            setTimeout(() => {
                window.syncWysiwygContent();
            }, 50);
        }
    }
}, true);

// Input listener on WYSIWYG surface for instant real-time sync
document.addEventListener('input', function(e) {
    if (e.target && (e.target.id === 'wysiwyg-editor-surface' || e.target.closest('#wysiwyg-editor-surface'))) {
        window.debouncedSyncWysiwyg(150);
    }
});

// HTML-to-Markdown Serializer for WYSIWYG
window.serializeWysiwygToMarkdown = function() {
    const surface = document.getElementById('wysiwyg-editor-surface');
    if (!surface) return null;

    function nodeToMd(node) {
        if (node.nodeType === Node.TEXT_NODE) {
            return node.textContent;
        }
        if (node.nodeType !== Node.ELEMENT_NODE) return '';

        const tag = node.tagName.toLowerCase();
        let inner = Array.from(node.childNodes).map(nodeToMd).join('');

        switch(tag) {
            case 'h1': return '# ' + inner.trim() + '\n\n';
            case 'h2': return '## ' + inner.trim() + '\n\n';
            case 'h3': return '### ' + inner.trim() + '\n\n';
            case 'h4': return '#### ' + inner.trim() + '\n\n';
            case 'h5': return '##### ' + inner.trim() + '\n\n';
            case 'h6': return '###### ' + inner.trim() + '\n\n';
            case 'p': return inner.trim() ? inner.trim() + '\n\n' : '\n';
            case 'strong':
            case 'b': return '**' + inner + '**';
            case 'em':
            case 'i': return '*' + inner + '*';
            case 'del':
            case 's':
            case 'strike': return '~~' + inner + '~~';
            case 'kbd': return '<kbd>' + inner + '</kbd>';
            case 'code':
                if (node.parentNode && node.parentNode.tagName.toLowerCase() === 'pre') {
                    return inner;
                }
                return '`' + inner + '`';
            case 'pre': {
                const codeNode = node.querySelector('code');
                const codeText = codeNode ? codeNode.textContent : node.textContent;
                const lang = node.getAttribute('data-lang') || '';
                return '```' + lang + '\n' + codeText.replace(/\n+$/, '') + '\n```\n\n';
            }
            case 'blockquote': return inner.split('\n').map(l => l ? '> ' + l : '>').join('\n') + '\n\n';
            case 'ul':
                return Array.from(node.children).map(li => {
                    const chk = li.querySelector('input[type=checkbox]');
                    if (chk) {
                        const isChecked = chk.checked || chk.hasAttribute('checked');
                        let text = nodeToMd(li).replace(/^\[[ xX]\]\s*/, '').trim();
                        return '- [' + (isChecked ? 'x' : ' ') + '] ' + text;
                    }
                    return '- ' + nodeToMd(li).trim();
                }).join('\n') + '\n\n';
            case 'ol':
                return Array.from(node.children).map((li, idx) => `${idx + 1}. ` + nodeToMd(li).trim()).join('\n') + '\n\n';
            case 'li': {
                let liText = '';
                for (const child of node.childNodes) {
                    if (child.nodeType === Node.ELEMENT_NODE && child.tagName.toLowerCase() === 'input' && child.type === 'checkbox') {
                        continue;
                    }
                    liText += nodeToMd(child);
                }
                return liText;
            }
            case 'hr': return '---\n\n';
            case 'a': {
                const href = node.getAttribute('href') || '';
                if (node.classList && node.classList.contains('heading-anchor')) return '';
                return '[' + inner + '](' + href + ')';
            }
            case 'img': return '![' + (node.getAttribute('alt') || '') + '](' + (node.getAttribute('src') || '') + ')';
            case 'table': {
                const rows = Array.from(node.querySelectorAll('tr'));
                if (rows.length === 0) return '';
                let mdTable = '';
                rows.forEach((row, rIdx) => {
                    const cells = Array.from(row.querySelectorAll('th, td')).map(c => nodeToMd(c).trim());
                    mdTable += '| ' + cells.join(' | ') + ' |\n';
                    if (rIdx === 0) {
                        mdTable += '| ' + cells.map(() => '---').join(' | ') + ' |\n';
                    }
                });
                return mdTable + '\n';
            }
            case 'details': {
                const summary = node.querySelector('summary');
                const summaryText = summary ? nodeToMd(summary).trim() : 'Details';
                const clone = node.cloneNode(true);
                const cloneSummary = clone.querySelector('summary');
                if (cloneSummary) cloneSummary.remove();
                const detailsContent = Array.from(clone.childNodes).map(nodeToMd).join('').trim();
                const isOpen = node.hasAttribute('open') ? ' open' : '';
                return `<details${isOpen}>\n<summary>${summaryText}</summary>\n\n${detailsContent}\n</details>\n\n`;
            }
            case 'summary': return inner;
            case 'div': {
                if (node.classList.contains('code-block-container')) {
                    const langLabel = node.querySelector('.code-lang-label');
                    const lang = langLabel ? langLabel.textContent.trim().toLowerCase() : '';
                    const codeContent = node.querySelector('.code-content pre') || node.querySelector('pre');
                    const text = codeContent ? codeContent.textContent : inner;
                    return '```' + (lang === 'text' ? '' : lang) + '\n' + text.replace(/\n+$/, '') + '\n```\n\n';
                }
                if (node.classList.contains('mdx-callout')) {
                    let type = 'info';
                    if (node.classList.contains('mdx-callout-warning')) type = 'warning';
                    else if (node.classList.contains('mdx-callout-danger')) type = 'danger';
                    else if (node.classList.contains('mdx-callout-tip')) type = 'tip';
                    else if (node.classList.contains('mdx-callout-note')) type = 'note';
                    return `<Callout type="${type}">\n${inner.trim()}\n</Callout>\n\n`;
                }
                if (node.classList.contains('mdx-card')) {
                    return `<Card>\n${inner.trim()}\n</Card>\n\n`;
                }
                if (node.classList.contains('mdx-steps')) {
                    return `<Steps>\n${inner.trim()}\n</Steps>\n\n`;
                }
                return inner ? inner + '\n' : '';
            }
            case 'span': {
                if (node.classList.contains('mdx-badge')) {
                    return `<Badge>${inner.trim()}</Badge>`;
                }
                return inner;
            }
            case 'section': {
                if (node.classList.contains('markdown-section')) {
                    return inner.trim() ? inner.trim() + '\n\n' : '';
                }
                return inner;
            }
            case 'input': return '';
            case 'button': return '';
            case 'br': return '\n';
            default: return inner;
        }
    }

    return Array.from(surface.childNodes).map(nodeToMd).join('').trim();
};
