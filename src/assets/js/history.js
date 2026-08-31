
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
