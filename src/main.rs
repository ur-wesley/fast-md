mod cli;
mod components;
mod i18n;
mod services;
mod state;
mod types;

use cli::CliArgs;
use components::{
    Editor, SettingsModal, Sidebar, StatusBar, TabBar, TitleBar, Toolbar, Viewer, ZenExitButton,
};
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use services::fs::{pick_file_async, pick_save_file_async, read_document_file};
use services::watcher::LiveFileWatcher;
use state::AppStore;
use std::env;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;
use types::{AppTheme, DocumentMode, Language, UpdateStatus};

static CLI_ARGS: OnceLock<CliArgs> = OnceLock::new();

const APP_STYLES: &str = include_str!("assets/style.css");

const HELPER_JS: &str = r#"
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

window.scrollToSection = function(id) {
    try {
        const el = document.getElementById(id);
        if (el) {
            el.scrollIntoView({ behavior: 'smooth', block: 'start' });
        }
    } catch(e) { console.error(e); }
};

// --- In-Document Search & Highlighting Engine ---
window._searchState = {
    matches: [],
    currentIndex: -1,
    query: ''
};

window.clearSearchHighlights = function() {
    try {
        const marks = document.querySelectorAll('mark.fastmd-search-match');
        marks.forEach(mark => {
            const parent = mark.parentNode;
            if (parent) {
                parent.replaceChild(document.createTextNode(mark.textContent), mark);
                parent.normalize();
            }
        });
        window._searchState.matches = [];
        window._searchState.currentIndex = -1;
        window._searchState.query = '';
        window.updateSearchCountUI(0, 0);
    } catch(e) { console.error(e); }
};

window.highlightSearchMatches = function(query) {
    try {
        window.clearSearchHighlights();
        if (!query || query.trim() === '') {
            return;
        }

        const root = document.querySelector('.app-main-viewer') || document.querySelector('.markdown-body');
        if (!root) return;

        window._searchState.query = query;
        const lowerQuery = query.toLowerCase();

        function walkTextNodes(node, callback) {
            if (node.nodeType === Node.TEXT_NODE) {
                callback(node);
            } else if (node.nodeType === Node.ELEMENT_NODE) {
                if (['SCRIPT', 'STYLE', 'BUTTON', 'INPUT', 'HEADER', 'NAV'].includes(node.tagName) || node.classList.contains('app-titlebar') || node.classList.contains('app-toolbar')) {
                    return;
                }
                Array.from(node.childNodes).forEach(child => walkTextNodes(child, callback));
            }
        }

        const textNodes = [];
        walkTextNodes(root, n => textNodes.push(n));

        const matches = [];
        textNodes.forEach(textNode => {
            const text = textNode.textContent;
            const lowerText = text.toLowerCase();
            let startIndex = 0;
            let index = lowerText.indexOf(lowerQuery, startIndex);

            if (index === -1) return;

            const fragment = document.createDocumentFragment();
            let lastIdx = 0;

            while (index !== -1) {
                if (index > lastIdx) {
                    fragment.appendChild(document.createTextNode(text.substring(lastIdx, index)));
                }

                const mark = document.createElement('mark');
                mark.className = 'fastmd-search-match';
                mark.textContent = text.substring(index, index + query.length);
                fragment.appendChild(mark);
                matches.push(mark);

                lastIdx = index + query.length;
                startIndex = lastIdx;
                index = lowerText.indexOf(lowerQuery, startIndex);
            }

            if (lastIdx < text.length) {
                fragment.appendChild(document.createTextNode(text.substring(lastIdx)));
            }

            if (textNode.parentNode) {
                textNode.parentNode.replaceChild(fragment, textNode);
            }
        });

        window._searchState.matches = matches;
        if (matches.length > 0) {
            window._searchState.currentIndex = 0;
            window.activateMatch(0);
        } else {
            window.updateSearchCountUI(0, 0);
        }
    } catch(e) { console.error(e); }
};

window.activateMatch = function(index) {
    try {
        const s = window._searchState;
        if (!s.matches || s.matches.length === 0) return;

        if (index < 0) index = s.matches.length - 1;
        if (index >= s.matches.length) index = 0;
        s.currentIndex = index;

        s.matches.forEach((m, idx) => {
            if (idx === index) {
                m.classList.add('active-match');
                m.scrollIntoView({ behavior: 'smooth', block: 'center' });
            } else {
                m.classList.remove('active-match');
            }
        });

        window.updateSearchCountUI(s.currentIndex + 1, s.matches.length);
    } catch(e) { console.error(e); }
};

window.searchNextMatch = function() {
    const s = window._searchState;
    if (s.matches && s.matches.length > 0) {
        window.activateMatch(s.currentIndex + 1);
    }
};

window.searchPrevMatch = function() {
    const s = window._searchState;
    if (s.matches && s.matches.length > 0) {
        window.activateMatch(s.currentIndex - 1);
    }
};

window.updateSearchCountUI = function(current, total) {
    try {
        const el = document.getElementById('search-match-count');
        if (el) {
            if (total === 0) {
                el.innerText = window._searchState && window._searchState.query ? '0 results' : '';
            } else {
                el.innerText = `${current} / ${total}`;
            }
        }
    } catch(e) { console.error(e); }
};

// Global Shortcut Interceptor (captures Ctrl+F / Cmd+F anywhere in the window)
function handleSearchShortcut(e) {
    if ((e.ctrlKey || e.metaKey) && (e.key === 'f' || e.key === 'F' || e.code === 'KeyF' || e.keyCode === 70)) {
        e.preventDefault();
        e.stopPropagation();
        e.stopImmediatePropagation();
        const input = document.getElementById('titlebar-search-input');
        if (input) {
            input.focus();
            input.select();
        }
        return false;
    }
}
window.addEventListener('keydown', handleSearchShortcut, true);

// --- Editor Textarea & WYSIWYG Helpers ---
function escapeRegex(str) {
    return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function findWordBoundaries(line, cursorOffset) {
    if (!line || line.length === 0) return { start: 0, end: 0, text: '' };

    let pos = cursorOffset;
    if (pos > 0 && pos === line.length && /[\w\u00C0-\u024F\-]/.test(line.charAt(pos - 1))) {
        pos--;
    } else if (pos < line.length && !/[\w\u00C0-\u024F\-]/.test(line.charAt(pos)) && pos > 0 && /[\w\u00C0-\u024F\-]/.test(line.charAt(pos - 1))) {
        pos--;
    }

    if (!/[\w\u00C0-\u024F\-]/.test(line.charAt(pos))) {
        return { start: cursorOffset, end: cursorOffset, text: '' };
    }

    let wStart = pos;
    while (wStart > 0 && /[\w\u00C0-\u024F\-]/.test(line.charAt(wStart - 1))) {
        wStart--;
    }

    let wEnd = pos;
    while (wEnd < line.length && /[\w\u00C0-\u024F\-]/.test(line.charAt(wEnd))) {
        wEnd++;
    }

    return {
        start: wStart,
        end: wEnd,
        text: line.substring(wStart, wEnd)
    };
}

// --- Editor History & Undo/Redo Engine ---
(function() {
    let history = [];
    let historyIndex = -1;
    const MAX_HISTORY = 150;
    let isHistoryNavigating = false;
    let pushTimeout = null;

    function initOrPushHistory(value, selStart, selEnd, immediate) {
        if (isHistoryNavigating) return;

        function record() {
            if (historyIndex >= 0 && history[historyIndex] && history[historyIndex].value === value) {
                history[historyIndex].selStart = selStart;
                history[historyIndex].selEnd = selEnd;
                return;
            }

            if (historyIndex < history.length - 1) {
                history = history.slice(0, historyIndex + 1);
            }

            history.push({ value, selStart, selEnd });
            if (history.length > MAX_HISTORY) {
                history.shift();
            }
            historyIndex = history.length - 1;
        }

        if (immediate) {
            if (pushTimeout) {
                clearTimeout(pushTimeout);
                pushTimeout = null;
            }
            record();
        } else {
            if (pushTimeout) clearTimeout(pushTimeout);
            pushTimeout = setTimeout(record, 200);
        }
    }

    window.pushEditorHistory = initOrPushHistory;

    window.editorUndo = function() {
        const wysiwygSurface = document.getElementById('wysiwyg-editor-surface');
        if (wysiwygSurface && (document.activeElement === wysiwygSurface || wysiwygSurface.contains(document.activeElement))) {
            wysiwygSurface.focus();
            document.execCommand('undo');
            wysiwygSurface.dispatchEvent(new Event('input', { bubbles: true }));
            if (window.updateToolbarActiveStates) window.updateToolbarActiveStates();
            return;
        }

        const ta = document.getElementById('source-markdown-textarea');
        if (!ta) {
            if (wysiwygSurface) {
                wysiwygSurface.focus();
                document.execCommand('undo');
                wysiwygSurface.dispatchEvent(new Event('input', { bubbles: true }));
                if (window.updateToolbarActiveStates) window.updateToolbarActiveStates();
            }
            return;
        }

        if (history.length === 0) {
            initOrPushHistory(ta.value, ta.selectionStart, ta.selectionEnd, true);
            return;
        }

        if (historyIndex === history.length - 1 && history[historyIndex] && history[historyIndex].value !== ta.value) {
            initOrPushHistory(ta.value, ta.selectionStart, ta.selectionEnd, true);
        }

        if (historyIndex > 0) {
            isHistoryNavigating = true;
            historyIndex--;
            const snap = history[historyIndex];
            ta.value = snap.value;
            ta.selectionStart = snap.selStart;
            ta.selectionEnd = snap.selEnd;
            ta.focus();
            ta.dispatchEvent(new Event('input', { bubbles: true }));
            isHistoryNavigating = false;
            if (window.updateToolbarActiveStates) window.updateToolbarActiveStates();
        }
    };

    window.editorRedo = function() {
        const wysiwygSurface = document.getElementById('wysiwyg-editor-surface');
        if (wysiwygSurface && (document.activeElement === wysiwygSurface || wysiwygSurface.contains(document.activeElement))) {
            wysiwygSurface.focus();
            document.execCommand('redo');
            wysiwygSurface.dispatchEvent(new Event('input', { bubbles: true }));
            if (window.updateToolbarActiveStates) window.updateToolbarActiveStates();
            return;
        }

        const ta = document.getElementById('source-markdown-textarea');
        if (!ta) {
            if (wysiwygSurface) {
                wysiwygSurface.focus();
                document.execCommand('redo');
                wysiwygSurface.dispatchEvent(new Event('input', { bubbles: true }));
                if (window.updateToolbarActiveStates) window.updateToolbarActiveStates();
            }
            return;
        }

        if (historyIndex < history.length - 1) {
            isHistoryNavigating = true;
            historyIndex++;
            const snap = history[historyIndex];
            ta.value = snap.value;
            ta.selectionStart = snap.selStart;
            ta.selectionEnd = snap.selEnd;
            ta.focus();
            ta.dispatchEvent(new Event('input', { bubbles: true }));
            isHistoryNavigating = false;
            if (window.updateToolbarActiveStates) window.updateToolbarActiveStates();
        }
    };

    document.addEventListener('input', (e) => {
        if (e.target && e.target.id === 'source-markdown-textarea') {
            initOrPushHistory(e.target.value, e.target.selectionStart, e.target.selectionEnd, false);
        }
    });

    document.addEventListener('keydown', (e) => {
        if ((e.ctrlKey || e.metaKey) && !e.altKey) {
            const ta = document.getElementById('source-markdown-textarea');
            if (ta && (document.activeElement === ta || ta.contains(document.activeElement))) {
                if (e.key === 'z' || e.key === 'Z') {
                    e.preventDefault();
                    if (e.shiftKey) {
                        window.editorRedo();
                    } else {
                        window.editorUndo();
                    }
                } else if (e.key === 'y' || e.key === 'Y') {
                    e.preventDefault();
                    window.editorRedo();
                }
            }
        }
    });
})();

window.wrapSourceSelection = function(prefix, suffix, defaultText) {
    const ta = document.getElementById('source-markdown-textarea');
    if (!ta) return;
    const start = ta.selectionStart;
    const end = ta.selectionEnd;
    const val = ta.value;

    const lineStart = val.lastIndexOf('\n', start - 1) + 1;
    const nextNl = val.indexOf('\n', start);
    const lineEnd = nextNl === -1 ? val.length : nextNl;
    const currentLine = val.substring(lineStart, lineEnd);
    const cursorInLine = start - lineStart;

    ta.focus();

    const isItalic = (prefix === '*' && suffix === '*');
    const isBold = (prefix === '**' && suffix === '**');

    const pLen = prefix.length;
    const sLen = suffix.length;

    // ==========================================
    // 1. Text IS Selected (start < end)
    // ==========================================
    if (start < end) {
        const selected = val.substring(start, end);

        // Case 1A: Selected text directly starts and ends with prefix & suffix (e.g. `*text*` or `**text**`)
        let isDirectMatch = false;
        if (isItalic) {
            if (selected.startsWith('*') && !selected.startsWith('**') && selected.endsWith('*') && !selected.endsWith('**') && selected.length >= 3) {
                isDirectMatch = true;
            }
        } else if (isBold) {
            if (selected.startsWith('**') && selected.endsWith('**') && selected.length >= 5) {
                isDirectMatch = true;
            }
        } else if (selected.startsWith(prefix) && selected.endsWith(suffix) && selected.length >= (pLen + sLen)) {
            isDirectMatch = true;
        }

        if (isDirectMatch) {
            const unwrapped = selected.substring(pLen, selected.length - sLen);
            ta.value = val.substring(0, start) + unwrapped + val.substring(end);
            ta.selectionStart = start;
            ta.selectionEnd = start + unwrapped.length;
            ta.dispatchEvent(new Event('input', { bubbles: true }));
            if (window.pushEditorHistory) window.pushEditorHistory(ta.value, ta.selectionStart, ta.selectionEnd, true);
            if (window.updateToolbarActiveStates) window.updateToolbarActiveStates();
            return;
        }

        // Case 1B: Text immediately outside selection matches prefix & suffix
        if (start >= pLen && end + sLen <= val.length) {
            const beforeSel = val.substring(start - pLen, start);
            const afterSel = val.substring(end, end + sLen);

            let isOuterMatch = (beforeSel === prefix && afterSel === suffix);

            if (isItalic) {
                const charBeforeBefore = start - pLen > 0 ? val.charAt(start - pLen - 1) : '';
                const charAtStart = start < val.length ? val.charAt(start) : '';
                const charAtEndMinusOne = end > 0 ? val.charAt(end - 1) : '';
                const charAfterAfter = end + sLen < val.length ? val.charAt(end + sLen) : '';

                if (charBeforeBefore === '*' || charAtStart === '*' || charAtEndMinusOne === '*' || charAfterAfter === '*') {
                    isOuterMatch = false; // It is part of bold **, NOT italic!
                }
            } else if (isBold) {
                const charBeforeBefore = start - pLen > 0 ? val.charAt(start - pLen - 1) : '';
                const charAfterAfter = end + sLen < val.length ? val.charAt(end + sLen) : '';
                if (charBeforeBefore === '*' || charAfterAfter === '*') {
                    isOuterMatch = false;
                }
            }

            if (isOuterMatch) {
                ta.value = val.substring(0, start - pLen) + selected + val.substring(end + sLen);
                ta.selectionStart = start - pLen;
                ta.selectionEnd = start - pLen + selected.length;
                ta.dispatchEvent(new Event('input', { bubbles: true }));
                if (window.pushEditorHistory) window.pushEditorHistory(ta.value, ta.selectionStart, ta.selectionEnd, true);
                if (window.updateToolbarActiveStates) window.updateToolbarActiveStates();
                return;
            }
        }

        // Case 1C: Nested formatting inside selected
        if (isItalic) {
            const italicRegex = /(?<!\*)\*([^*\n]+?)\*(?!\*)/g;
            if (italicRegex.test(selected)) {
                const cleaned = selected.replace(/(?<!\*)\*([^*\n]+?)\*(?!\*)/g, '$1');
                ta.value = val.substring(0, start) + cleaned + val.substring(end);
                ta.selectionStart = start;
                ta.selectionEnd = start + cleaned.length;
                ta.dispatchEvent(new Event('input', { bubbles: true }));
                if (window.pushEditorHistory) window.pushEditorHistory(ta.value, ta.selectionStart, ta.selectionEnd, true);
                if (window.updateToolbarActiveStates) window.updateToolbarActiveStates();
                return;
            }
        } else if (isBold) {
            const boldRegex = /\*\*([^*\n]+?)\*\*/g;
            if (boldRegex.test(selected)) {
                const cleaned = selected.replace(/\*\*([^*\n]+?)\*\*/g, '$1');
                ta.value = val.substring(0, start) + cleaned + val.substring(end);
                ta.selectionStart = start;
                ta.selectionEnd = start + cleaned.length;
                ta.dispatchEvent(new Event('input', { bubbles: true }));
                if (window.pushEditorHistory) window.pushEditorHistory(ta.value, ta.selectionStart, ta.selectionEnd, true);
                if (window.updateToolbarActiveStates) window.updateToolbarActiveStates();
                return;
            }
        } else if (prefix === suffix) {
            const escPfx = escapeRegex(prefix);
            const regex = new RegExp(escPfx + '([^' + escapeRegex(prefix.charAt(0)) + '\n]+?)' + escPfx, 'g');
            if (regex.test(selected)) {
                const cleaned = selected.replace(regex, '$1');
                ta.value = val.substring(0, start) + cleaned + val.substring(end);
                ta.selectionStart = start;
                ta.selectionEnd = start + cleaned.length;
                ta.dispatchEvent(new Event('input', { bubbles: true }));
                if (window.pushEditorHistory) window.pushEditorHistory(ta.value, ta.selectionStart, ta.selectionEnd, true);
                if (window.updateToolbarActiveStates) window.updateToolbarActiveStates();
                return;
            }
        }

        // Case 1D: Wrap selection cleanly
        const wrapped = prefix + selected + suffix;
        ta.value = val.substring(0, start) + wrapped + val.substring(end);
        ta.selectionStart = start + pLen;
        ta.selectionEnd = start + pLen + selected.length;
        ta.dispatchEvent(new Event('input', { bubbles: true }));
        if (window.pushEditorHistory) window.pushEditorHistory(ta.value, ta.selectionStart, ta.selectionEnd, true);
        if (window.updateToolbarActiveStates) window.updateToolbarActiveStates();
        return;
    }

    // ==========================================
    // 2. Collapsed Cursor (start === end)
    // ==========================================
    let foundMatch = null;

    if (isItalic) {
        // Match ONLY standalone single asterisks: (?<!\*)\*([^*\n]+?)\*(?!\*)
        const regex = /(?<!\*)\*([^*\n]+?)\*(?!\*)/g;
        let match;
        while ((match = regex.exec(currentLine)) !== null) {
            const mStart = match.index;
            const mEnd = match.index + match[0].length;
            if (cursorInLine >= mStart && cursorInLine <= mEnd) {
                foundMatch = {
                    start: lineStart + mStart,
                    end: lineStart + mEnd,
                    inner: match[1]
                };
                break;
            }
        }
    } else if (isBold) {
        // Match bold: \*\*([^*\n]+?)\*\*
        const regex = /\*\*([^*\n]+?)\*\*/g;
        let match;
        while ((match = regex.exec(currentLine)) !== null) {
            const mStart = match.index;
            const mEnd = match.index + match[0].length;
            if (cursorInLine >= mStart && cursorInLine <= mEnd) {
                foundMatch = {
                    start: lineStart + mStart,
                    end: lineStart + mEnd,
                    inner: match[1]
                };
                break;
            }
        }
    } else if (prefix === suffix) {
        const escPfx = escapeRegex(prefix);
        const firstCh = escapeRegex(prefix.charAt(0));
        const regexStr = escPfx + '([^' + firstCh + '\n]+?)' + escPfx;
        const regex = new RegExp(regexStr, 'g');
        let match;
        while ((match = regex.exec(currentLine)) !== null) {
            const mStart = match.index;
            const mEnd = match.index + match[0].length;
            if (cursorInLine >= mStart && cursorInLine <= mEnd) {
                foundMatch = {
                    start: lineStart + mStart,
                    end: lineStart + mEnd,
                    inner: match[1]
                };
                break;
            }
        }
    }

    if (foundMatch) {
        // Cursor inside format -> UNWRAP (remove enclosing markers)
        ta.value = val.substring(0, foundMatch.start) + foundMatch.inner + val.substring(foundMatch.end);
        const newCursor = Math.max(lineStart, Math.min(ta.value.length, start - pLen));
        ta.selectionStart = newCursor;
        ta.selectionEnd = newCursor;
        ta.dispatchEvent(new Event('input', { bubbles: true }));
        if (window.pushEditorHistory) window.pushEditorHistory(ta.value, ta.selectionStart, ta.selectionEnd, true);
        if (window.updateToolbarActiveStates) window.updateToolbarActiveStates();
        return;
    }

    // 3. Not inside existing wrapper: check if cursor is on/inside a word to wrap!
    const word = findWordBoundaries(currentLine, cursorInLine);
    if (word.text.length > 0) {
        const wordAbsStart = lineStart + word.start;
        const wordAbsEnd = lineStart + word.end;
        const wrappedWord = prefix + word.text + suffix;

        ta.value = val.substring(0, wordAbsStart) + wrappedWord + val.substring(wordAbsEnd);
        const newCursor = start + pLen;
        ta.selectionStart = newCursor;
        ta.selectionEnd = newCursor;
        ta.dispatchEvent(new Event('input', { bubbles: true }));
        if (window.pushEditorHistory) window.pushEditorHistory(ta.value, ta.selectionStart, ta.selectionEnd, true);
        if (window.updateToolbarActiveStates) window.updateToolbarActiveStates();
        return;
    }

    // 4. Insert new wrapper with placeholder selected
    const replacement = prefix + defaultText + suffix;
    ta.value = val.substring(0, start) + replacement + val.substring(end);
    ta.selectionStart = start + pLen;
    ta.selectionEnd = start + pLen + defaultText.length;
    ta.dispatchEvent(new Event('input', { bubbles: true }));
    if (window.pushEditorHistory) window.pushEditorHistory(ta.value, ta.selectionStart, ta.selectionEnd, true);
    if (window.updateToolbarActiveStates) window.updateToolbarActiveStates();
};

window.insertSourceLinePrefix = function(prefix) {
    const ta = document.getElementById('source-markdown-textarea');
    if (!ta) return;
    const start = ta.selectionStart;
    const val = ta.value;

    const lineStart = val.lastIndexOf('\n', start - 1) + 1;
    const nextNl = val.indexOf('\n', start);
    const lineEnd = nextNl === -1 ? val.length : nextNl;
    const currentLine = val.substring(lineStart, lineEnd);

    const isHeading = prefix.startsWith('#');
    const isListOrQuote = prefix.startsWith('-') || prefix.startsWith('1.') || prefix.startsWith('>');

    let oldPrefixMatch = null;
    let oldPrefixLen = 0;

    if (isHeading) {
        const m = currentLine.match(/^#{1,6}\s*/);
        if (m) {
            oldPrefixMatch = m[0];
            oldPrefixLen = m[0].length;
        }
    } else if (isListOrQuote) {
        const m = currentLine.match(/^(?:-\s*\[[ xX]\]\s*|[-*+]\s+|\d+\.\s+|>+\s*)/);
        if (m) {
            oldPrefixMatch = m[0];
            oldPrefixLen = m[0].length;
        }
    }

    let newLine = '';
    let cursorShift = 0;

    if (oldPrefixMatch !== null) {
        const contentAfter = currentLine.substring(oldPrefixLen);
        if (oldPrefixMatch.trim() === prefix.trim()) {
            // Same prefix clicked -> TOGGLE OFF (remove prefix)
            newLine = contentAfter;
            cursorShift = -oldPrefixLen;
        } else {
            // Different prefix -> REPLACE old prefix with new prefix
            newLine = prefix + contentAfter;
            cursorShift = prefix.length - oldPrefixLen;
        }
    } else {
        // No prefix -> ADD prefix
        newLine = prefix + currentLine;
        cursorShift = prefix.length;
    }

    ta.focus();
    ta.value = val.substring(0, lineStart) + newLine + val.substring(lineEnd);
    const newCursor = Math.max(lineStart, Math.min(ta.value.length, start + cursorShift));
    ta.selectionStart = newCursor;
    ta.selectionEnd = newCursor;
    ta.dispatchEvent(new Event('input', { bubbles: true }));
    if (window.pushEditorHistory) window.pushEditorHistory(ta.value, ta.selectionStart, ta.selectionEnd, true);
    if (window.updateToolbarActiveStates) {
        window.updateToolbarActiveStates();
    }
};

window.insertSourceSnippet = function(snippet) {
    const ta = document.getElementById('source-markdown-textarea');
    if (!ta) return;
    ta.focus();
    if (document.execCommand) {
        document.execCommand('insertText', false, snippet);
    } else {
        const start = ta.selectionStart;
        const end = ta.selectionEnd;
        const val = ta.value;
        ta.value = val.substring(0, start) + snippet + val.substring(end);
        ta.selectionStart = start + snippet.length;
        ta.selectionEnd = start + snippet.length;
        ta.dispatchEvent(new Event('input', { bubbles: true }));
        if (window.pushEditorHistory) window.pushEditorHistory(ta.value, ta.selectionStart, ta.selectionEnd, true);
    }
};

window.handleTextareaTab = function(e) {
    const ta = document.getElementById('source-markdown-textarea');
    if (!ta) return;
    e.preventDefault();
    if (document.execCommand) {
        document.execCommand('insertText', false, '  ');
    } else {
        const start = ta.selectionStart;
        const end = ta.selectionEnd;
        const val = ta.value;
        ta.value = val.substring(0, start) + '  ' + val.substring(end);
        ta.selectionStart = start + 2;
        ta.selectionEnd = start + 2;
        ta.dispatchEvent(new Event('input', { bubbles: true }));
        if (window.pushEditorHistory) window.pushEditorHistory(ta.value, ta.selectionStart, ta.selectionEnd, true);
    }
};

// WYSIWYG Actions
window.formatWysiwyg = function(cmd, val) {
    const el = document.getElementById('wysiwyg-editor-surface');
    if (!el) return;
    el.focus();
    document.execCommand(cmd, false, val || null);
    if (window.updateToolbarActiveStates) window.updateToolbarActiveStates();
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
};

window.formatWysiwygBlockquote = function() {
    const el = document.getElementById('wysiwyg-editor-surface');
    if (!el) return;
    el.focus();
    document.execCommand('formatBlock', false, 'blockquote');
};

window.insertWysiwygCodeBlock = function() {
    const el = document.getElementById('wysiwyg-editor-surface');
    if (!el) return;
    el.focus();
    document.execCommand('insertHTML', false, '<pre><code>// Code snippet\n</code></pre><p><br></p>');
};

window.insertWysiwygTable = function() {
    const el = document.getElementById('wysiwyg-editor-surface');
    if (!el) return;
    el.focus();
    document.execCommand('insertHTML', false, '<table><thead><tr><th>Header 1</th><th>Header 2</th></tr></thead><tbody><tr><td>Value 1</td><td>Value 2</td></tr></tbody></table><p><br></p>');
};

window.insertWysiwygCallout = function(type) {
    const el = document.getElementById('wysiwyg-editor-surface');
    if (!el) return;
    el.focus();
    document.execCommand('insertHTML', false, `<div class="mdx-callout mdx-callout-${type || 'info'}"><p>Callout note description</p></div><p><br></p>`);
};

window.insertWysiwygTaskList = function() {
    const el = document.getElementById('wysiwyg-editor-surface');
    if (!el) return;
    el.focus();
    document.execCommand('insertHTML', false, '<ul class="task-list"><li><input type="checkbox"> Task item</li></ul><p><br></p>');
};

window.promptWysiwygLink = function() {
    const url = prompt('Enter URL:');
    if (url) {
        document.execCommand('createLink', false, url);
    }
};

window.promptWysiwygImage = function() {
    const url = prompt('Enter Image URL:');
    if (url) {
        document.execCommand('insertImage', false, url);
    }
};

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
            case 'code':
                if (node.parentNode && node.parentNode.tagName.toLowerCase() === 'pre') {
                    return inner;
                }
                return '`' + inner + '`';
            case 'pre': return '```\n' + inner.trim() + '\n```\n\n';
            case 'blockquote': return inner.split('\n').map(l => l ? '> ' + l : '>').join('\n') + '\n\n';
            case 'ul':
                return Array.from(node.children).map(li => {
                    const chk = li.querySelector('input[type=checkbox]');
                    if (chk) {
                        return '- [' + (chk.checked ? 'x' : ' ') + '] ' + nodeToMd(li).replace(/^\[.\]\s*/, '').trim();
                    }
                    return '- ' + nodeToMd(li).trim();
                }).join('\n') + '\n\n';
            case 'ol':
                return Array.from(node.children).map((li, idx) => `${idx + 1}. ` + nodeToMd(li).trim()).join('\n') + '\n\n';
            case 'li': return inner;
            case 'hr': return '---\n\n';
            case 'a': return '[' + inner + '](' + (node.getAttribute('href') || '') + ')';
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
            case 'div': {
                if (node.classList.contains('mdx-callout')) {
                    let type = 'info';
                    if (node.classList.contains('mdx-callout-warning')) type = 'warning';
                    else if (node.classList.contains('mdx-callout-danger')) type = 'error';
                    else if (node.classList.contains('mdx-callout-tip')) type = 'tip';
                    return `<Callout type="${type}">\n${inner.trim()}\n</Callout>\n\n`;
                }
                return inner ? inner + '\n' : '';
            }
            case 'br': return '\n';
            default: return inner;
        }
    }

    return Array.from(surface.childNodes).map(nodeToMd).join('').trim();
};

// --- WYSIWYG & Editor Toolbar Active State Highlighting Engine ---
(function() {
    function updateToolbarActiveStates() {
        const buttons = document.querySelectorAll('.editor-toolbar [data-tool]');
        if (!buttons || buttons.length === 0) return;

        const activeTools = new Set();

        // 1. Check WYSIWYG surface cursor/selection
        const wysiwygSurface = document.getElementById('wysiwyg-editor-surface');
        const selection = window.getSelection();

        if (wysiwygSurface && selection && selection.rangeCount > 0 && wysiwygSurface.contains(selection.anchorNode)) {
            let node = selection.anchorNode;
            if (node.nodeType === Node.TEXT_NODE) {
                node = node.parentNode;
            }

            while (node && node !== wysiwygSurface) {
                const tag = node.tagName ? node.tagName.toLowerCase() : '';
                if (tag === 'b' || tag === 'strong' || (node.style && (node.style.fontWeight === 'bold' || parseInt(node.style.fontWeight, 10) >= 700))) {
                    activeTools.add('bold');
                }
                if (tag === 'i' || tag === 'em' || (node.style && node.style.fontStyle === 'italic')) {
                    activeTools.add('italic');
                }
                if (tag === 's' || tag === 'del' || tag === 'strike' || (node.style && node.style.textDecoration && node.style.textDecoration.includes('line-through'))) {
                    activeTools.add('strikethrough');
                }
                if (tag === 'code' && (!node.parentNode || node.parentNode.tagName.toLowerCase() !== 'pre')) {
                    activeTools.add('code');
                }
                if (tag === 'pre' || (tag === 'code' && node.parentNode && node.parentNode.tagName.toLowerCase() === 'pre')) {
                    activeTools.add('codeblock');
                }
                if (tag === 'h1') activeTools.add('h1');
                if (tag === 'h2') activeTools.add('h2');
                if (tag === 'h3') activeTools.add('h3');
                if (tag === 'blockquote') activeTools.add('quote');
                if (tag === 'ul') {
                    if (node.classList && node.classList.contains('task-list')) {
                        activeTools.add('task');
                    } else {
                        activeTools.add('ul');
                    }
                }
                if (tag === 'ol') activeTools.add('ol');
                if (tag === 'table' || tag === 'td' || tag === 'th' || tag === 'tr') activeTools.add('table');
                if (tag === 'a') activeTools.add('link');
                if (tag === 'div' && node.classList && node.classList.contains('mdx-callout')) activeTools.add('callout');

                node = node.parentNode;
            }

            try {
                if (document.queryCommandState('bold')) activeTools.add('bold');
                if (document.queryCommandState('italic')) activeTools.add('italic');
                if (document.queryCommandState('strikeThrough')) activeTools.add('strikethrough');
                if (document.queryCommandState('insertUnorderedList')) activeTools.add('ul');
                if (document.queryCommandState('insertOrderedList')) activeTools.add('ol');
            } catch(e) {}
        }

        // 2. Check Raw Markdown Source Textarea cursor/selection
        const textarea = document.getElementById('source-markdown-textarea');
        if (textarea && (document.activeElement === textarea || textarea.contains(document.activeElement))) {
            const start = textarea.selectionStart;
            const text = textarea.value;

            // Get current line
            const lastNl = text.lastIndexOf('\n', start - 1);
            const lineStart = lastNl === -1 ? 0 : lastNl + 1;
            const nextNl = text.indexOf('\n', start);
            const lineEnd = nextNl === -1 ? text.length : nextNl;
            const currentLine = text.substring(lineStart, lineEnd);
            const cursorInLine = start - lineStart;
            const trimmedLine = currentLine.trim();

            if (trimmedLine.startsWith('# ') || /^#\s/.test(trimmedLine)) activeTools.add('h1');
            else if (trimmedLine.startsWith('## ') || /^##\s/.test(trimmedLine)) activeTools.add('h2');
            else if (trimmedLine.startsWith('### ') || /^###\s/.test(trimmedLine)) activeTools.add('h3');
            else if (trimmedLine.startsWith('> ') || trimmedLine === '>') activeTools.add('quote');
            else if (/^-\s*\[[ xX]\]/.test(trimmedLine)) activeTools.add('task');
            else if (/^[-*+]\s+/.test(trimmedLine)) activeTools.add('ul');
            else if (/^\d+\.\s+/.test(trimmedLine)) activeTools.add('ol');
            else if (trimmedLine.startsWith('|') || trimmedLine.includes('|')) activeTools.add('table');

            // Inline Bold detection in line: **...** or __...__
            const boldRegex = /(?:\*\*([^*]+?)\*\*|__([^_]+?)__)/g;
            let m;
            while ((m = boldRegex.exec(currentLine)) !== null) {
                const mStart = m.index;
                const mEnd = m.index + m[0].length;
                if (cursorInLine >= mStart && cursorInLine <= mEnd) {
                    activeTools.add('bold');
                }
            }

            // Inline Italic detection in line: *...* (not **) or _..._ (not __)
            const italicRegex = /(?:(?<!\*)\*([^*]+?)\*(?!\*)|(?<!_)_([^_]+?)_(?!_))/g;
            while ((m = italicRegex.exec(currentLine)) !== null) {
                const mStart = m.index;
                const mEnd = m.index + m[0].length;
                if (cursorInLine >= mStart && cursorInLine <= mEnd) {
                    activeTools.add('italic');
                }
            }

            // Inline Strikethrough detection in line: ~~...~~
            const strikeRegex = /~~([^~]+?)~~/g;
            while ((m = strikeRegex.exec(currentLine)) !== null) {
                const mStart = m.index;
                const mEnd = m.index + m[0].length;
                if (cursorInLine >= mStart && cursorInLine <= mEnd) {
                    activeTools.add('strikethrough');
                }
            }

            // Inline Code detection in line: `...`
            const codeRegex = /`([^`]+?)`/g;
            while ((m = codeRegex.exec(currentLine)) !== null) {
                const mStart = m.index;
                const mEnd = m.index + m[0].length;
                if (cursorInLine >= mStart && cursorInLine <= mEnd) {
                    activeTools.add('code');
                }
            }

            // Inline Link detection in line: [...](...)
            const linkRegex = /\[([^\]]+?)\]\(([^)]+?)\)/g;
            while ((m = linkRegex.exec(currentLine)) !== null) {
                const mStart = m.index;
                const mEnd = m.index + m[0].length;
                if (cursorInLine >= mStart && cursorInLine <= mEnd) {
                    activeTools.add('link');
                }
            }

            // Check if cursor is inside code fence (```...```)
            const textBefore = text.substring(0, start);
            const fenceMatches = textBefore.match(/```/g);
            if (fenceMatches && fenceMatches.length % 2 === 1) {
                activeTools.add('codeblock');
            }

            // Check if cursor is inside callout (<Callout...</Callout>)
            const lastCalloutOpen = textBefore.lastIndexOf('<Callout');
            const lastCalloutClose = textBefore.lastIndexOf('</Callout>');
            if (lastCalloutOpen !== -1 && lastCalloutOpen > lastCalloutClose) {
                activeTools.add('callout');
            }
        }

        // Apply active states to tool buttons
        buttons.forEach(btn => {
            const tool = btn.getAttribute('data-tool');
            if (activeTools.has(tool)) {
                btn.classList.add('active-tool');
                btn.setAttribute('data-active', 'true');
            } else {
                btn.classList.remove('active-tool');
                btn.removeAttribute('data-active');
            }
        });
    }

    window.updateToolbarActiveStates = updateToolbarActiveStates;

    document.addEventListener('selectionchange', () => {
        requestAnimationFrame(updateToolbarActiveStates);
    });

    ['keyup', 'mouseup', 'click', 'input', 'focus'].forEach(evtName => {
        document.addEventListener(evtName, (e) => {
            if (e.target && (e.target.id === 'wysiwyg-editor-surface' || e.target.id === 'source-markdown-textarea' || e.target.closest('#wysiwyg-editor-surface'))) {
                requestAnimationFrame(updateToolbarActiveStates);
            }
        });
    });
})();

// --- Synchronized Scroll Engine for Split Mode ---
(function() {
    let isSyncingFromEditor = false;
    let isSyncingFromPreview = false;

    window.onEditorSourceScroll = function() {
        const textarea = document.getElementById('source-markdown-textarea');
        const gutter = document.getElementById('source-line-gutter');
        const preview = document.getElementById('split-preview-scroll-area');

        if (textarea && gutter) {
            gutter.scrollTop = textarea.scrollTop;
        }

        if (isSyncingFromPreview) {
            isSyncingFromPreview = false;
            return;
        }

        if (textarea && preview) {
            const editorMax = textarea.scrollHeight - textarea.clientHeight;
            const previewMax = preview.scrollHeight - preview.clientHeight;

            if (editorMax > 0 && previewMax > 0) {
                const ratio = textarea.scrollTop / editorMax;
                isSyncingFromEditor = true;
                preview.scrollTop = ratio * previewMax;
            }
        }
    };

    window.onSplitPreviewScroll = function() {
        if (isSyncingFromEditor) {
            isSyncingFromEditor = false;
            return;
        }

        const textarea = document.getElementById('source-markdown-textarea');
        const gutter = document.getElementById('source-line-gutter');
        const preview = document.getElementById('split-preview-scroll-area');

        if (textarea && preview) {
            const editorMax = textarea.scrollHeight - textarea.clientHeight;
            const previewMax = preview.scrollHeight - preview.clientHeight;

            if (editorMax > 0 && previewMax > 0) {
                const ratio = preview.scrollTop / previewMax;
                isSyncingFromPreview = true;
                textarea.scrollTop = ratio * editorMax;
                if (gutter) {
                    gutter.scrollTop = textarea.scrollTop;
                }
            }
        }
    };
})();

// --- Reading & Scroll Progress Engine ---
(function() {
    let _scrollTicking = false;
    let _lastActiveHeadingId = '';

    function updateScrollProgress() {
        _scrollTicking = false;
        try {
            const scrollArea = document.getElementById('viewer-scroll-area');
            if (!scrollArea) return;

            const scrollTop = scrollArea.scrollTop;
            const scrollHeight = scrollArea.scrollHeight;
            const clientHeight = scrollArea.clientHeight;
            const maxScroll = scrollHeight - clientHeight;

            let progress = 0;
            if (maxScroll > 0) {
                progress = Math.min(100, Math.max(0, (scrollTop / maxScroll) * 100));
            }

            const viewerBar = document.getElementById('viewer-scroll-progress-bar');
            if (viewerBar) {
                viewerBar.style.width = progress + '%';
            }

            // Find active heading based on viewport offset
            const headings = scrollArea.querySelectorAll('.doc-heading, h1[id], h2[id], h3[id], h4[id], h5[id], h6[id]');
            if (!headings || headings.length === 0) return;

            let activeHeadingId = headings[0].id;
            const offsetThreshold = 110;

            for (let i = 0; i < headings.length; i++) {
                const h = headings[i];
                const rect = h.getBoundingClientRect();
                const containerRect = scrollArea.getBoundingClientRect();
                const relativeTop = rect.top - containerRect.top;

                if (relativeTop <= offsetThreshold) {
                    activeHeadingId = h.id;
                } else {
                    break;
                }
            }

            if (scrollTop + clientHeight >= scrollHeight - 15) {
                activeHeadingId = headings[headings.length - 1].id;
            }

            if (activeHeadingId !== _lastActiveHeadingId) {
                _lastActiveHeadingId = activeHeadingId;

                const tocItems = document.querySelectorAll('.sidebar-toc-container .toc-item');
                let foundActive = false;
                let activeElem = null;

                tocItems.forEach((item) => {
                    const hId = item.getAttribute('data-heading-id');
                    if (hId === activeHeadingId) {
                        item.classList.add('active-toc-item');
                        item.classList.remove('passed-toc-item', 'future-toc-item');
                        foundActive = true;
                        activeElem = item;
                    } else if (!foundActive) {
                        item.classList.add('passed-toc-item');
                        item.classList.remove('active-toc-item', 'future-toc-item');
                    } else {
                        item.classList.add('future-toc-item');
                        item.classList.remove('active-toc-item', 'passed-toc-item');
                    }
                });

                if (activeElem && !window._userIsHoveringToc) {
                    const tocContainer = document.querySelector('.sidebar-toc-container');
                    if (tocContainer) {
                        const itemTop = activeElem.offsetTop;
                        const itemBottom = itemTop + activeElem.offsetHeight;
                        const containerTop = tocContainer.scrollTop;
                        const containerBottom = containerTop + tocContainer.clientHeight;

                        if (itemTop < containerTop + 30 || itemBottom > containerBottom - 30) {
                            activeElem.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
                        }
                    }
                }
            }
        } catch(e) {
            console.error('Progress update error:', e);
        }
    }

    window.onViewerScroll = function() {
        if (!_scrollTicking) {
            window.requestAnimationFrame(updateScrollProgress);
            _scrollTicking = true;
        }
    };

    window.addEventListener('resize', () => {
        if (!_scrollTicking) {
            window.requestAnimationFrame(updateScrollProgress);
            _scrollTicking = true;
        }
    });
})();
"#;

fn resolve_cli_path(raw_path: Option<&PathBuf>) -> Option<PathBuf> {
    raw_path.and_then(|p| {
        if p.is_absolute() && p.exists() {
            Some(p.clone())
        } else if let Ok(current_dir) = env::current_dir() {
            let combined = current_dir.join(p);
            if combined.exists() {
                Some(combined)
            } else if p.exists() {
                Some(p.clone())
            } else {
                None
            }
        } else {
            None
        }
    })
}

fn main() {
    let args = CliArgs::parse_safe();

    if args.register {
        if services::association::register_file_associations() {
            println!("Successfully registered Fast-MD in Windows Explorer and Default Apps.");
        } else {
            eprintln!("Failed to register file associations.");
        }
        return;
    }

    if args.unregister {
        if services::association::unregister_file_associations() {
            println!("Successfully unregistered Fast-MD file associations.");
        } else {
            eprintln!("Failed to unregister file associations.");
        }
        return;
    }

    // Auto-register file associations on startup in background
    std::thread::spawn(|| {
        let _ = services::association::register_file_associations();
    });

    let _ = CLI_ARGS.set(args);

    let config = Config::new()
        .with_window(
            WindowBuilder::new()
                .with_title("Fast-MD Viewer & Editor")
                .with_decorations(false)
                .with_transparent(true)
                .with_inner_size(dioxus::desktop::LogicalSize::new(1180.0, 800.0))
                .with_min_inner_size(dioxus::desktop::LogicalSize::new(640.0, 420.0)),
        )
        .with_background_color((0, 0, 0, 0))
        .with_custom_head(format!("<script src=\"https://cdn.tailwindcss.com\"></script><style>{APP_STYLES}</style><script>{HELPER_JS}</script>"));

    dioxus::LaunchBuilder::desktop().with_cfg(config).launch(App);
}

#[component]
fn App() -> Element {
    let cli_args = CLI_ARGS.get().cloned().unwrap_or(CliArgs {
        path: None,
        zen: false,
        theme: None,
        lang: None,
        register: false,
        unregister: false,
    });

    let cli_theme = cli_args.theme.as_deref().and_then(|t| match t.to_lowercase().as_str() {
        "light" => Some(AppTheme::Light),
        "midnight" => Some(AppTheme::Midnight),
        "nord" => Some(AppTheme::Nord),
        "solarized" | "solarized-dark" => Some(AppTheme::SolarizedDark),
        "latte" | "catppuccin-latte" => Some(AppTheme::CatppuccinLatte),
        "frappe" | "frappé" | "catppuccin-frappe" | "catppuccin-frappé" => Some(AppTheme::CatppuccinFrappe),
        "macchiato" | "catppuccin-macchiato" => Some(AppTheme::CatppuccinMacchiato),
        "mocha" | "catppuccin" | "catppuccin-mocha" => Some(AppTheme::CatppuccinMocha),
        "dark" => Some(AppTheme::Dark),
        _ => None,
    });

    let cli_lang = cli_args.lang.as_deref().and_then(|l| match l.to_lowercase().as_str() {
        "de" | "german" | "deutsch" => Some(Language::De),
        "en" | "english" => Some(Language::En),
        _ => None,
    });

    let resolved_path = resolve_cli_path(cli_args.path.as_ref());
    let initial_zen = cli_args.zen;

    // Central application state store
    let mut store = use_signal(move || {
        AppStore::new_with_options(resolved_path.as_deref(), cli_theme, cli_lang, initial_zen)
    });

    // Dynamically apply OS native glass / acrylic / mica effect based on active theme
    use_effect(move || {
        let current_theme = store().theme;
        let win = dioxus::desktop::window();

        #[cfg(target_os = "windows")]
        {
            let is_dark = current_theme.is_dark();
            let _ = window_vibrancy::apply_mica(&**win, Some(is_dark));
        }

        #[cfg(target_os = "macos")]
        {
            let is_dark = current_theme.is_dark();
            let material = if is_dark {
                window_vibrancy::NSVisualEffectMaterial::FullScreenUI
            } else {
                window_vibrancy::NSVisualEffectMaterial::WindowBackground
            };
            let _ = window_vibrancy::apply_vibrancy(&**win, material, None, None);
        }
    });

    // File watcher setup attached to central store (respects auto_reload setting)
    let _watcher_task = use_coroutine(move |_: UnboundedReceiver<()>| {
        to_owned![store];
        async move {
            if let Ok((mut watcher, _tx)) = LiveFileWatcher::new() {
                loop {
                    tokio::time::sleep(Duration::from_millis(600)).await;

                    let s = store();
                    if s.settings.auto_reload {
                        if let Some(active_tab) = s.active_tab() {
                            if let Some(ref path) = active_tab.path {
                                let _ = watcher.watch_path(path);
                            }
                        }

                        while let Ok(changed_path) = watcher.receiver.try_recv() {
                            if let Ok(new_content) = read_document_file(&changed_path) {
                                store.write().update_file_content_if_modified(&changed_path, &new_content);
                            }
                        }
                    }
                }
            }
        }
    });

    // Background GitHub release auto-checker on application startup
    let _update_checker_task = use_coroutine(move |_: UnboundedReceiver<()>| {
        to_owned![store];
        async move {
            let should_check = store().settings.auto_check_updates;
            if should_check {
                tokio::time::sleep(Duration::from_secs(2)).await;
                let res = tokio::task::spawn_blocking(services::updater::check_github_release).await;
                if let Ok(Ok(Some(release))) = res {
                    store.write().set_update_status(UpdateStatus::Available(release));
                }
            }
        }
    });

    let store_read = store();
    let current_theme_class = store_read.theme.as_str();
    let is_zen = store_read.is_zen;
    let is_full_width = store_read.is_full_width;
    let zoom_level = store_read.zoom_level;
    let show_sidebar = store_read.show_sidebar;
    let show_settings_modal = store_read.show_settings_modal;
    let document_mode = store_read.mode;

    let root_style = store_read.primary_color.as_ref().map_or_else(String::new, |color| {
        format!("--accent: {color}; --accent-hover: {color}; --accent-glow: {color}40;")
    });

    let active_tab = store_read.active_tab().cloned().unwrap_or_else(|| state::AppStore::default().tabs.remove(0));

    rsx! {
        div {
            class: format!("app-root {current_theme_class}"),
            style: "{root_style}",
            tabindex: 0,
            onkeydown: move |evt| {
                let key = evt.key();
                let ctrl = evt.modifiers().ctrl();
                let shift = evt.modifiers().shift();

                if key == Key::Escape {
                    let mut s = store.write();
                    if s.show_settings_modal {
                        s.set_settings_modal(false);
                    } else if s.is_zen {
                        s.set_zen(false);
                    } else if s.show_search {
                        s.show_search = false;
                    }
                } else if ctrl && (key == Key::Character(",".to_string()) || key == Key::Character("<".to_string())) {
                    store.write().toggle_settings_modal();
                } else if ctrl && (key == Key::Character("s".to_string()) || key == Key::Character("S".to_string())) {
                    if shift {
                        // Save As
                        spawn(async move {
                            let s = store();
                            if let Some(active) = s.active_tab() {
                                let title = active.title.clone();
                                if let Some(path) = pick_save_file_async(&title).await {
                                    let id = active.id;
                                    let _ = store.write().save_tab_with_path(id, path);
                                }
                            }
                        });
                    } else {
                        // Save
                        spawn(async move {
                            let s = store();
                            if let Some(active) = s.active_tab() {
                                if let Some(ref _p) = active.path {
                                    let _ = store.write().save_active_tab();
                                } else {
                                    let title = active.title.clone();
                                    if let Some(path) = pick_save_file_async(&title).await {
                                        let id = active.id;
                                        let _ = store.write().save_tab_with_path(id, path);
                                    }
                                }
                            }
                        });
                    }
                } else if (shift && evt.modifiers().alt() && (key == Key::Character("f".to_string()) || key == Key::Character("F".to_string())))
                    || (ctrl && shift && (key == Key::Character("i".to_string()) || key == Key::Character("I".to_string()))) {
                    store.write().format_active_tab();
                } else if ctrl && (key == Key::Character("e".to_string()) || key == Key::Character("E".to_string())) {
                    store.write().cycle_mode();
                } else if ctrl && key == Key::Character("o".to_string()) {
                    spawn(async move {
                        if let Some(path) = pick_file_async().await {
                            store.write().open_file_from_path(path);
                        }
                    });
                } else if ctrl && (key == Key::Character("f".to_string()) || key == Key::Character("F".to_string())) && !shift {
                    dioxus::prelude::document::eval(
                        r"
                        const input = document.getElementById('titlebar-search-input');
                        if (input) { input.focus(); input.select(); }
                        ",
                    );
                } else if ctrl && (key == Key::Character("=".to_string()) || key == Key::Character("+".to_string())) {
                    store.write().zoom_in();
                } else if ctrl && key == Key::Character("-".to_string()) {
                    store.write().zoom_out();
                } else if ctrl && key == Key::Character("0".to_string()) {
                    store.write().reset_zoom();
                } else if ctrl && shift && (key == Key::Character("F".to_string()) || key == Key::Character("f".to_string())) {
                    store.write().toggle_zen();
                } else if ctrl && key == Key::Character("t".to_string()) {
                    spawn(async move {
                        if let Some(path) = pick_file_async().await {
                            store.write().open_file_from_path(path);
                        } else {
                            store.write().new_empty_tab();
                        }
                    });
                } else if ctrl && key == Key::Character("w".to_string()) {
                    let current_id = store().active_tab_id;
                    store.write().close_tab(current_id);
                }
            },

            // Floating Zen Exit Button (visible only in Zen mode)
            if is_zen {
                ZenExitButton {
                    language: store_read.language,
                    on_exit: move |()| {
                        store.write().set_zen(false);
                    },
                }
            }

            // Custom Window Title Bar (hidden in Zen mode)
            if !is_zen {
                TitleBar {
                    store: store,
                }
            }

            // Top Toolbar (hidden in Zen mode)
            if !is_zen {
                Toolbar {
                    store: store,
                }
            }

            // Tab Bar (hidden in Zen mode)
            if !is_zen {
                TabBar {
                    store: store,
                }
            }

            // Main Workspace Layout
            div {
                class: if is_zen { "app-workspace-body zen-active" } else { "app-workspace-body" },

                // Sidebar
                if show_sidebar && !is_zen {
                    Sidebar {
                        store: store,
                        on_select_heading: move |id| {
                            dioxus::prelude::document::eval(&format!("window.scrollToSection && window.scrollToSection('{id}');"));
                        },
                    }
                }

                // Main Content Area: Viewer OR Editor based on DocumentMode
                if document_mode == DocumentMode::View {
                    Viewer {
                        document: active_tab.parsed.clone(),
                        is_full_width: is_full_width,
                        zoom_level: zoom_level,
                        sticky_headers: store_read.sticky_headers,
                        language: store_read.language,
                    }
                } else {
                    Editor {
                        store: store,
                        mode: document_mode,
                        document: active_tab.parsed.clone(),
                        raw_content: active_tab.content.clone(),
                        is_full_width: is_full_width,
                        zoom_level: zoom_level,
                        sticky_headers: store_read.sticky_headers,
                        language: store_read.language,
                    }
                }
            }

            // Bottom Status Bar (hidden in Zen mode)
            if !is_zen {
                StatusBar {
                    title: active_tab.title,
                    file_path: active_tab.path,
                    document: active_tab.parsed,
                    raw_content: active_tab.content,
                    mode: document_mode,
                    is_dirty: active_tab.is_dirty,
                    zoom_level: zoom_level,
                    language: store_read.language,
                    on_cycle_mode: move |()| store.write().cycle_mode(),
                }
            }

            // Settings Modal Dialog
            if show_settings_modal {
                SettingsModal {
                    store: store,
                }
            }
        }
    }
}


