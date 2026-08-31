
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

function insertTextareaSpaces(ta, text) {
    if (document.execCommand && ta === document.activeElement) {
        document.execCommand('insertText', false, text);
        ta.dispatchEvent(new Event('input', { bubbles: true }));
        if (window.pushEditorHistory) window.pushEditorHistory(ta.value, ta.selectionStart, ta.selectionEnd, true);
    } else {
        const start = ta.selectionStart;
        const end = ta.selectionEnd;
        const val = ta.value;
        ta.value = val.substring(0, start) + text + val.substring(end);
        ta.selectionStart = start + text.length;
        ta.selectionEnd = start + text.length;
        ta.dispatchEvent(new Event('input', { bubbles: true }));
        if (window.pushEditorHistory) window.pushEditorHistory(ta.value, ta.selectionStart, ta.selectionEnd, true);
    }
}

window.handleSourceTab = function(e) {
    const ta = document.getElementById('source-markdown-textarea');
    if (!ta || e.target !== ta) return;
    if (e.key !== 'Tab' || e.ctrlKey || e.metaKey || e.altKey || e.isComposing) return;

    if (e.shiftKey) {
        const start = ta.selectionStart;
        const val = ta.value;
        const { lineStart, line, cursorInLine } = getLineInfo(val, start);
        const leading = line.match(/^(\s*)/)[1];
        const remove = Math.min(2, leading.length, cursorInLine);
        if (remove === 0) return;
        e.preventDefault();
        const newLine = line.substring(remove);
        const newVal = val.substring(0, lineStart) + newLine + val.substring(lineStart + line.length);
        applyTextareaEdit(ta, newVal, Math.max(lineStart, start - remove));
        return;
    }

    e.preventDefault();
    insertTextareaSpaces(ta, '  ');
};

function getLineInfo(text, pos) {
    const lineStart = text.lastIndexOf('\n', pos - 1) + 1;
    const nextNl = text.indexOf('\n', pos);
    const lineEnd = nextNl === -1 ? text.length : nextNl;
    const line = text.substring(lineStart, lineEnd);
    return {
        lineStart,
        lineEnd,
        line,
        cursorInLine: pos - lineStart,
    };
}

function isInsideFencedCode(text, pos) {
    const before = text.substring(0, pos);
    let count = 0;
    let idx = 0;
    while ((idx = before.indexOf('```', idx)) !== -1) {
        count++;
        idx += 3;
    }
    idx = 0;
    while ((idx = before.indexOf('~~~', idx)) !== -1) {
        count++;
        idx += 3;
    }
    return count % 2 === 1;
}

const LINE_PREFIX_RE = /^(\s*)(?:-\s*\[[ xX]\]\s*|[-*+]\s+|\d+\.\s+|>+\s*)/;

function revealSourceCaret(ta) {
    if (!ta) return;
    requestAnimationFrame(function() {
        const pos = ta.selectionStart;
        const val = ta.value;
        if (pos === 0 || val.charAt(pos - 1) === '\n') {
            ta.scrollLeft = 0;
        }

        const lineHeight = parseFloat(getComputedStyle(ta).lineHeight) || 21;
        const lineIndex = val.substring(0, pos).split('\n').length - 1;
        const lineTop = lineIndex * lineHeight;
        const lineBottom = lineTop + lineHeight;
        const viewTop = ta.scrollTop;
        const viewBottom = viewTop + ta.clientHeight;

        if (lineTop < viewTop) {
            ta.scrollTop = lineTop;
        } else if (lineBottom > viewBottom) {
            ta.scrollTop = lineBottom - ta.clientHeight;
        }
    });
}

function applyTextareaEdit(ta, newValue, cursorPos) {
    ta.value = newValue;
    ta.selectionStart = cursorPos;
    ta.selectionEnd = cursorPos;
    ta.dispatchEvent(new Event('input', { bubbles: true }));
    if (window.pushEditorHistory) window.pushEditorHistory(ta.value, ta.selectionStart, ta.selectionEnd, true);
    if (window.updateToolbarActiveStates) window.updateToolbarActiveStates();
}

function handleListEnter(ta, val, start) {
    const { lineStart, lineEnd, line, cursorInLine } = getLineInfo(val, start);
    const match = line.match(LINE_PREFIX_RE);
    if (!match) return false;

    const indent = match[1];
    const fullPrefix = match[0];
    const marker = fullPrefix.slice(indent.length);
    const contentStart = fullPrefix.length;
    const contentBefore = line.substring(contentStart, cursorInLine);
    const contentAfter = line.substring(cursorInLine);
    const contentOnly = line.substring(contentStart);

    if (contentOnly.trim() === '') {
        const newVal = val.substring(0, lineStart) + indent + val.substring(lineEnd);
        applyTextareaEdit(ta, newVal, lineStart + indent.length);
        return true;
    }

    let nextMarker = marker;
    const numMatch = marker.match(/^(\d+)\.\s+$/);
    if (numMatch) {
        nextMarker = `${parseInt(numMatch[1], 10) + 1}. `;
    } else if (/^[-*+]\s*\[[ xX]\]\s*$/.test(marker)) {
        nextMarker = '- [ ] ';
    }

    const currentLine = fullPrefix + contentBefore;
    const nextLine = indent + nextMarker + contentAfter;
    const insert = currentLine + '\n' + nextLine;
    const newVal = val.substring(0, lineStart) + insert + val.substring(lineEnd);
    const newCursor = lineStart + currentLine.length + 1 + indent.length + nextMarker.length;
    applyTextareaEdit(ta, newVal, newCursor);
    return true;
}

function splitTableRow(line) {
    let trimmed = line.trim();
    if (trimmed.startsWith('|')) trimmed = trimmed.slice(1);
    if (trimmed.endsWith('|')) trimmed = trimmed.slice(0, -1);

    const cells = [];
    let current = '';
    const chars = trimmed.split('');
    for (let i = 0; i < chars.length; i++) {
        if (chars[i] === '\\' && chars[i + 1] === '|') {
            current += '|';
            i++;
        } else if (chars[i] === '|') {
            cells.push(current.trim());
            current = '';
        } else {
            current += chars[i];
        }
    }
    cells.push(current.trim());
    return cells;
}

function isTableRow(line) {
    const trimmed = line.trim();
    if (!trimmed || !trimmed.includes('|')) return false;
    return trimmed.startsWith('|') || trimmed.endsWith('|') || (trimmed.match(/\|/g) || []).length >= 2;
}

function isTableDelimiterRow(line) {
    const trimmed = line.trim();
    if (!trimmed.includes('|') || !trimmed.includes('-')) return false;
    const cells = splitTableRow(trimmed);
    if (cells.length === 0) return false;
    return cells.every((c) => {
        const t = c.trim();
        return t.length > 0 && /^:?-{1,}:?$/.test(t.replace(/\s/g, ''));
    });
}

function makeTableRow(cells, empty) {
    const body = cells.map((c) => (empty ? ' ' : c)).join(' | ');
    return `| ${body} |`;
}

function handleTableEnter(ta, val, start) {
    const { lineStart, lineEnd, line } = getLineInfo(val, start);
    if (!isTableRow(line)) return false;

    const cells = splitTableRow(line);
    const colCount = cells.length;
    if (colCount === 0) return false;

    if (cells.every((c) => c.trim() === '')) {
        let removeEnd = lineEnd;
        if (removeEnd < val.length && val[removeEnd] === '\n') removeEnd++;
        const newVal = val.substring(0, lineStart) + val.substring(removeEnd);
        applyTextareaEdit(ta, newVal, lineStart);
        return true;
    }

    const afterLine = lineEnd < val.length && val[lineEnd] === '\n' ? lineEnd + 1 : lineEnd;
    const nextNl = val.indexOf('\n', afterLine);
    const nextLine = val.substring(afterLine, nextNl === -1 ? val.length : nextNl);
    const isDelimiter = isTableDelimiterRow(line);

    if (!isDelimiter && isTableDelimiterRow(nextLine)) {
        const emptyRow = makeTableRow(Array(colCount).fill(''), true);
        const insertAt = nextNl === -1 ? val.length : nextNl;
        const suffix = val.substring(insertAt);
        const prefix = val.substring(0, insertAt);
        const insert = (suffix.startsWith('\n') ? '\n' : '\n') + emptyRow;
        const newVal = prefix + insert + suffix;
        const newCursor = prefix.length + insert.length - emptyRow.length + 2;
        applyTextareaEdit(ta, newVal, newCursor);
        return true;
    }

    if (!isDelimiter && !isTableDelimiterRow(nextLine)) {
        const delim = makeTableRow(Array(colCount).fill('---'), false);
        const emptyRow = makeTableRow(Array(colCount).fill(''), true);
        const suffix = val.substring(lineEnd);
        const prefix = val.substring(0, lineEnd);
        const insert = (suffix.startsWith('\n') ? '' : '\n') + delim + '\n' + emptyRow;
        const newVal = prefix + insert + suffix;
        const newCursor = prefix.length + insert.length - emptyRow.length + 2;
        applyTextareaEdit(ta, newVal, newCursor);
        return true;
    }

    const emptyRow = makeTableRow(Array(colCount).fill(''), true);
    const suffix = val.substring(lineEnd);
    const prefix = val.substring(0, lineEnd);
    const insert = (suffix.startsWith('\n') ? '\n' : '\n') + emptyRow;
    const newVal = prefix + insert + suffix;
    const newCursor = prefix.length + insert.length - emptyRow.length + 2;
    applyTextareaEdit(ta, newVal, newCursor);
    return true;
}

window.handleSourceEnter = function(e) {
    const ta = document.getElementById('source-markdown-textarea');
    if (!ta || e.target !== ta) return;
    if (e.key !== 'Enter' || e.shiftKey || e.ctrlKey || e.metaKey || e.altKey || e.isComposing) return;

    const start = ta.selectionStart;
    const end = ta.selectionEnd;
    if (start !== end) return;

    const val = ta.value;
    if (isInsideFencedCode(val, start)) return;

    if (handleListEnter(ta, val, start)) {
        e.preventDefault();
        revealSourceCaret(ta);
        return;
    }

    if (handleTableEnter(ta, val, start)) {
        e.preventDefault();
        revealSourceCaret(ta);
    }
};

window.handleSourceEnterKeyup = function(e) {
    const ta = document.getElementById('source-markdown-textarea');
    if (!ta || e.target !== ta) return;
    if (e.key !== 'Enter' || e.ctrlKey || e.metaKey || e.altKey || e.isComposing) return;
    revealSourceCaret(ta);
};

document.addEventListener('keydown', function(e) {
    if (window.handleSourceTab) window.handleSourceTab(e);
    if (window.handleSourceEnter) window.handleSourceEnter(e);
}, true);

document.addEventListener('keyup', function(e) {
    if (window.handleSourceEnterKeyup) window.handleSourceEnterKeyup(e);
}, true);
