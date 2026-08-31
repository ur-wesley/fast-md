// --- Global Shortcut Listener Engine ---
(function() {
    window.__globalShortcutHandler = null;

    // Keys that shouldn't be prevented when editing inside input/textarea unless specifically handled
    function isStandardEditingKey(e) {
        const key = e.key.toLowerCase();
        const hasCtrlOrMeta = e.ctrlKey || e.metaKey;
        if (!hasCtrlOrMeta || e.altKey) return false;
        return (key === 'c' || key === 'v' || key === 'x' || key === 'a' || key === 'z' || key === 'y');
    }

    function isInputFocused() {
        const active = document.activeElement;
        if (!active) return false;
        const tag = active.tagName ? active.tagName.toLowerCase() : '';
        return tag === 'input' || tag === 'textarea' || active.isContentEditable;
    }

    window.addEventListener('keydown', function(e) {
        const hasModifier = e.ctrlKey || e.metaKey || e.altKey;
        const isEscape = e.key === 'Escape';
        const isFKey = /^F\d+$/i.test(e.key);

        if (!hasModifier && !isEscape && !isFKey) {
            return;
        }

        // If recording shortcut in preferences, let the recorder handle it
        if (window.__recordingShortcut) {
            return;
        }

        const payload = {
            key: e.key,
            code: e.code,
            ctrlKey: e.ctrlKey,
            metaKey: e.metaKey,
            altKey: e.altKey,
            shiftKey: e.shiftKey
        };

        const inInput = isInputFocused();
        const standardEditing = isStandardEditingKey(e);

        // Prevent browser defaults for common app shortcut keys like Ctrl+S, Ctrl+O, Ctrl+W, Ctrl+F, Ctrl+P, Ctrl+T, Ctrl+=, Ctrl+-, Ctrl+0, Ctrl+E, Ctrl+,
        const keyLower = e.key.toLowerCase();
        if ((e.ctrlKey || e.metaKey) && (
            keyLower === 's' ||
            keyLower === 'o' ||
            keyLower === 'w' ||
            keyLower === 't' ||
            keyLower === 'e' ||
            keyLower === 'b' ||
            keyLower === 'f' ||
            keyLower === ',' ||
            keyLower === '<' ||
            keyLower === '=' ||
            keyLower === '+' ||
            keyLower === '-' ||
            keyLower === '0'
        )) {
            e.preventDefault();
        } else if (e.shiftKey && e.altKey && (keyLower === 'f' || keyLower === 'i')) {
            e.preventDefault();
        } else if (isEscape) {
            // Escape can be handled globally
        }

        if (window.__globalShortcutHandler) {
            window.__globalShortcutHandler(payload);
        }

        window.dispatchEvent(new CustomEvent('fastmd:shortcut', { detail: payload }));
    }, true); // Use capture phase to intercept before children
})();
