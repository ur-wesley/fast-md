
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
