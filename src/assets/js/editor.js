
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
