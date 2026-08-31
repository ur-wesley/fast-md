
// --- Synchronized Scroll & View Mode Scroll Preservation Engine ---
(function() {
    let isSyncingFromEditor = false;
    let isSyncingFromPreview = false;
    let _isRestoringScroll = false;
    const _tabScrollRatioMap = new Map();
    let _activeTabId = '0';

    const SCROLLER_IDS = [
        'viewer-scroll-area',
        'split-preview-scroll-area',
        'wysiwyg-scroll-area',
        'source-markdown-textarea',
    ];

    function usedLineHeightPx(style) {
        const raw = style.lineHeight;
        const n = parseFloat(raw);
        if (!raw || raw === 'normal' || !(n >= 4)) {
            const fs = parseFloat(style.fontSize) || 16;
            return fs * 1.625;
        }
        return n;
    }

    function textareaContentHeight(textarea, style) {
        const padTop = parseFloat(style.paddingTop) || 0;
        const padBottom = parseFloat(style.paddingBottom) || 0;
        return Math.max(0, textarea.scrollHeight - padTop - padBottom);
    }

    function syncGutterToTextarea(textarea) {
        const inner = document.getElementById('source-line-gutter-inner');
        if (!inner || !textarea) return;

        const style = window.getComputedStyle(textarea);
        const padTop = parseFloat(style.paddingTop) || 0;
        inner.style.paddingTop = '0px';
        inner.style.paddingBottom = '0px';

        const y = padTop - textarea.scrollTop;
        inner.style.transform = 'translateY(' + y + 'px)';
    }

    let _gutterCurrentLine = 0;

    function caretLineNumber(ta) {
        const pos = Math.max(0, Math.min(ta.selectionStart || 0, ta.value.length));
        if (pos === 0) return 1;
        let n = 1;
        let from = 0;
        while (true) {
            const nl = ta.value.indexOf('\n', from);
            if (nl === -1 || nl >= pos) break;
            n++;
            from = nl + 1;
        }
        return n;
    }

    function updateGutterCurrentLine() {
        const ta = document.getElementById('source-markdown-textarea');
        const inner = document.getElementById('source-line-gutter-inner');
        if (!ta || !inner) {
            _gutterCurrentLine = 0;
            return;
        }
        const line = caretLineNumber(ta);
        if (line === _gutterCurrentLine) {
            const still = inner.querySelector('.gutter-line.is-current');
            if (still && still.getAttribute('data-line') === String(line)) return;
        }
        const prev = inner.querySelector('.gutter-line.is-current');
        if (prev) prev.classList.remove('is-current');
        const next = inner.querySelector('.gutter-line[data-line="' + line + '"]');
        if (next) next.classList.add('is-current');
        _gutterCurrentLine = line;
    }

    let _gutterMirror = null;

    function getOrCreateGutterMirror() {
        if (_gutterMirror && _gutterMirror.tagName !== 'TEXTAREA') {
            _gutterMirror.remove();
            _gutterMirror = null;
        }
        if (!_gutterMirror) {
            _gutterMirror = document.createElement('textarea');
            _gutterMirror.id = 'source-gutter-measure-mirror';
            _gutterMirror.setAttribute('aria-hidden', 'true');
            _gutterMirror.tabIndex = -1;
            _gutterMirror.readOnly = true;
            _gutterMirror.rows = 1;
            _gutterMirror.style.cssText = [
                'position:absolute',
                'visibility:hidden',
                'pointer-events:none',
                'overflow:hidden',
                'height:0',
                'min-height:0',
                'max-height:none',
                'top:0',
                'left:-9999px',
                'z-index:-1',
                'margin:0',
                'resize:none',
                'border:none',
            ].join(';');
            document.body.appendChild(_gutterMirror);
        }
        return _gutterMirror;
    }

    function copyTextareaTypo(dest, style) {
        dest.style.font = style.font;
        dest.style.fontSize = style.fontSize;
        dest.style.fontFamily = style.fontFamily;
        dest.style.fontWeight = style.fontWeight;
        dest.style.fontStyle = style.fontStyle;
        dest.style.lineHeight = style.lineHeight;
        dest.style.letterSpacing = style.letterSpacing;
        dest.style.wordSpacing = style.wordSpacing;
        dest.style.tabSize = style.tabSize;
        dest.style.MozTabSize = style.tabSize;
        dest.style.whiteSpace = style.whiteSpace;
        dest.style.wordBreak = style.wordBreak;
        dest.style.overflowWrap = style.overflowWrap;
        dest.style.hyphens = style.hyphens;
    }

    function prepareGutterMirror(mirror, textarea, style) {
        copyTextareaTypo(mirror, style);
        mirror.style.boxSizing = 'border-box';
        mirror.style.width = textarea.clientWidth + 'px';
        mirror.style.paddingLeft = style.paddingLeft;
        mirror.style.paddingRight = style.paddingRight;
        mirror.style.paddingTop = '0px';
        mirror.style.paddingBottom = '0px';
        mirror.style.borderWidth = '0';
        mirror.style.height = '0px';
        mirror.style.overflow = 'hidden';
    }

    function measureProbeLineHeight(mirror, style) {
        const savedWhiteSpace = mirror.style.whiteSpace;
        const savedWordBreak = mirror.style.wordBreak;
        const savedOverflowWrap = mirror.style.overflowWrap;
        mirror.style.whiteSpace = 'pre';
        mirror.style.wordBreak = 'normal';
        mirror.style.overflowWrap = 'normal';
        mirror.value = 'x\n'.repeat(8);
        const probe = mirror.scrollHeight / 8;
        mirror.style.whiteSpace = savedWhiteSpace;
        mirror.style.wordBreak = savedWordBreak;
        mirror.style.overflowWrap = savedOverflowWrap;
        return probe > 0 ? probe : usedLineHeightPx(style);
    }

    function measureWrapCount(mirror, lineText, probeLh) {
        mirror.value = lineText.length > 0 ? lineText : ' ';
        return Math.max(1, Math.round(mirror.scrollHeight / probeLh));
    }

    function applyGutterFlexRow(el, grow, style) {
        el.style.fontSize = style.fontSize;
        el.style.fontFamily = style.fontFamily;
        el.style.height = '';
        el.style.lineHeight = '';
        el.style.flexGrow = String(grow);
        el.style.flexShrink = '1';
        el.style.flexBasis = '0';
        el.style.minHeight = '0';
    }

    function syncGutterLineHeights(textarea) {
        if (!textarea) return;
        const inner = document.getElementById('source-line-gutter-inner');
        if (!inner) return;

        const gutterLines = inner.querySelectorAll('.gutter-line');
        if (gutterLines.length === 0) return;

        const style = window.getComputedStyle(textarea);
        const contentH = textareaContentHeight(textarea, style);

        inner.style.fontSize = style.fontSize;
        inner.style.fontFamily = style.fontFamily;
        inner.style.paddingTop = '0px';
        inner.style.paddingBottom = '0px';
        inner.style.lineHeight = '';
        inner.style.display = 'flex';
        inner.style.flexDirection = 'column';
        inner.style.height = contentH + 'px';
        inner.style.boxSizing = 'border-box';

        const lineWrap = textarea.dataset.lineWrap === '1';
        const count = gutterLines.length;

        if (!lineWrap) {
            for (let i = 0; i < count; i++) {
                applyGutterFlexRow(gutterLines[i], 1, style);
            }
            return;
        }

        if (textarea.clientWidth <= 0) return;

        const mirror = getOrCreateGutterMirror();
        prepareGutterMirror(mirror, textarea, style);
        mirror.style.whiteSpace = style.whiteSpace || 'pre-wrap';
        mirror.style.wordBreak = style.wordBreak || 'break-all';
        mirror.style.overflowWrap = style.overflowWrap;

        const probeLh = measureProbeLineHeight(mirror, style);
        const lines = textarea.value.split('\n');

        for (let i = 0; i < count; i++) {
            const lineText = i < lines.length ? lines[i] : '';
            const grow = measureWrapCount(mirror, lineText, probeLh);
            applyGutterFlexRow(gutterLines[i], grow, style);
        }
    }

    window.syncGutterLineHeights = syncGutterLineHeights;
    window.syncGutterToTextarea = syncGutterToTextarea;

    function resolveActiveTabId() {
        if (_activeTabId !== null && _activeTabId !== undefined) {
            return _activeTabId;
        }
        const elWithTabId = document.querySelector('[data-tab-id]');
        if (elWithTabId) {
            const id = elWithTabId.getAttribute('data-tab-id');
            if (id) return id;
        }
        return '0';
    }

    function resolveScroller() {
        const viewer = document.getElementById('viewer-scroll-area');
        if (viewer) return viewer;
        const splitPreview = document.getElementById('split-preview-scroll-area');
        if (splitPreview) return splitPreview;
        const wysiwyg = document.getElementById('wysiwyg-scroll-area');
        if (wysiwyg) return wysiwyg;
        const textarea = document.getElementById('source-markdown-textarea');
        if (textarea) return textarea;
        return null;
    }

    function scrollRatio(el) {
        if (!el) return null;
        const max = el.scrollHeight - el.clientHeight;
        if (max > 0) {
            return Math.max(0, Math.min(1, el.scrollTop / max));
        }
        return 0;
    }

    function applyScrollRatio(el, ratio) {
        if (!el || ratio === undefined || ratio === null) return;
        const max = el.scrollHeight - el.clientHeight;
        if (max > 0) {
            el.scrollTop = ratio * max;
        }
    }

    function tabScrollRatios(tabId) {
        const tId = String(tabId);
        if (!_tabScrollRatioMap.has(tId)) {
            _tabScrollRatioMap.set(tId, {});
        }
        return _tabScrollRatioMap.get(tId);
    }

    function saveScrollerScroll(scrollerId, tabId) {
        if (_isRestoringScroll) return;
        const el = document.getElementById(scrollerId);
        if (!el) return;
        const tId = (tabId !== undefined && tabId !== null) ? String(tabId) : resolveActiveTabId();
        const ratios = tabScrollRatios(tId);
        const ratio = scrollRatio(el);
        if (ratio !== null) {
            ratios[scrollerId] = ratio;
        }
        _activeTabId = tId;
    }

    function saveCurrentScroll(tabId) {
        if (_isRestoringScroll) return;
        const tId = (tabId !== undefined && tabId !== null) ? String(tabId) : resolveActiveTabId();
        const ratios = tabScrollRatios(tId);
        for (let i = 0; i < SCROLLER_IDS.length; i++) {
            const id = SCROLLER_IDS[i];
            const el = document.getElementById(id);
            const ratio = scrollRatio(el);
            if (el && ratio !== null) {
                ratios[id] = ratio;
            }
        }
        _activeTabId = tId;
    }

    window.saveCurrentScroll = saveCurrentScroll;

    function isSourceEditorFocused() {
        const textarea = document.getElementById('source-markdown-textarea');
        return textarea && document.activeElement === textarea;
    }

    function isWysiwygFocused() {
        const surface = document.getElementById('wysiwyg-editor-surface');
        if (!surface) return false;
        const active = document.activeElement;
        return active === surface || surface.contains(active);
    }

    function isCaretAtLineStart(text, pos) {
        return pos === 0 || text.charAt(pos - 1) === '\n';
    }

    function installTextareaValueGuard(textarea) {
        if (!textarea || textarea.dataset.valueGuardInstalled === '1') return;
        const nativeDescriptor = Object.getOwnPropertyDescriptor(
            HTMLTextAreaElement.prototype,
            'value'
        );
        if (!nativeDescriptor || !nativeDescriptor.set || !nativeDescriptor.get) return;

        textarea.dataset.valueGuardInstalled = '1';
        const nativeSet = nativeDescriptor.set;
        const nativeGet = nativeDescriptor.get;

        Object.defineProperty(textarea, 'value', {
            configurable: true,
            enumerable: nativeDescriptor.enumerable,
            get: function() {
                return nativeGet.call(this);
            },
            set: function(nextValue) {
                if (document.activeElement !== this) {
                    nativeSet.call(this, nextValue);
                    return;
                }

                const scrollTop = this.scrollTop;
                const scrollLeft = this.scrollLeft;
                const selStart = this.selectionStart;
                const selEnd = this.selectionEnd;

                _isRestoringScroll = true;
                try {
                    nativeSet.call(this, nextValue);

                    const len = this.value.length;
                    const start = Math.min(selStart, len);
                    const end = Math.min(selEnd, len);
                    try {
                        this.setSelectionRange(start, end);
                    } catch (err) {
                        // ignore invalid range
                    }

                    this.scrollTop = scrollTop;
                    if (isCaretAtLineStart(this.value, start)) {
                        this.scrollLeft = 0;
                    } else {
                        this.scrollLeft = scrollLeft;
                    }

                    syncGutterLineHeights(this);
                    syncGutterToTextarea(this);
                } finally {
                    _isRestoringScroll = false;
                }
            },
        });
    }

    function restoreScrollState(tabId) {
        if (isSourceEditorFocused() || isWysiwygFocused()) {
            return;
        }
        if (tabId !== undefined && tabId !== null) {
            _activeTabId = String(tabId);
        }
        const tId = resolveActiveTabId();
        const ratios = tabScrollRatios(tId);

        function applyScroll() {
            if (isSourceEditorFocused() || isWysiwygFocused()) {
                return;
            }
            _isRestoringScroll = true;
            for (let i = 0; i < SCROLLER_IDS.length; i++) {
                const id = SCROLLER_IDS[i];
                if (ratios[id] === undefined) continue;
                applyScrollRatio(document.getElementById(id), ratios[id]);
            }
            syncGutterToTextarea(document.getElementById('source-markdown-textarea'));
            if (window.onViewerScroll) {
                window.onViewerScroll();
            }
            _isRestoringScroll = false;
        }

        applyScroll();
        requestAnimationFrame(applyScroll);
        setTimeout(applyScroll, 30);
        setTimeout(applyScroll, 80);
    }

    window.restoreScrollState = restoreScrollState;

    function paneScrollT(el) {
        const max = el.scrollHeight - el.clientHeight;
        if (max <= 0) return 0;
        return Math.min(1, Math.max(0, el.scrollTop / max));
    }

    function setPaneScrollT(el, t) {
        const max = el.scrollHeight - el.clientHeight;
        if (max > 0) {
            el.scrollTop = Math.min(1, Math.max(0, t)) * max;
        }
    }

    window.onEditorSourceScroll = function() {
        const textarea = document.getElementById('source-markdown-textarea');
        const preview = document.getElementById('split-preview-scroll-area');

        syncGutterToTextarea(textarea);

        if (isSyncingFromPreview) {
            return;
        }

        if (textarea && preview) {
            isSyncingFromEditor = true;
            setPaneScrollT(preview, paneScrollT(textarea));
            isSyncingFromEditor = false;
        }

        saveScrollerScroll('source-markdown-textarea', _activeTabId);
        if (preview) saveScrollerScroll('split-preview-scroll-area', _activeTabId);
        if (window.onViewerScroll) window.onViewerScroll();
    };

    window.onSplitPreviewScroll = function() {
        if (isSyncingFromEditor) {
            return;
        }

        const textarea = document.getElementById('source-markdown-textarea');
        const preview = document.getElementById('split-preview-scroll-area');

        if (textarea && preview && !isSourceEditorFocused()) {
            isSyncingFromPreview = true;
            setPaneScrollT(textarea, paneScrollT(preview));
            syncGutterToTextarea(textarea);
            isSyncingFromPreview = false;
        }

        saveScrollerScroll('split-preview-scroll-area', _activeTabId);
        if (window.onViewerScroll) window.onViewerScroll();
    };

    window.bindEditorScroll = function(tabId) {
        if (tabId !== undefined && tabId !== null) {
            _activeTabId = String(tabId);
        }
        const textarea = document.getElementById('source-markdown-textarea');
        const preview = document.getElementById('split-preview-scroll-area');
        const wysiwyg = document.getElementById('wysiwyg-scroll-area');

        if (textarea) {
            installTextareaValueGuard(textarea);
            if (textarea.dataset.editorScrollBound !== '1') {
                textarea.dataset.editorScrollBound = '1';
                textarea.addEventListener('scroll', function() {
                    window.onEditorSourceScroll();
                });
                ['keyup', 'click', 'input'].forEach(function(evtName) {
                    textarea.addEventListener(evtName, function() {
                        updateGutterCurrentLine();
                        if (evtName === 'input') {
                            syncGutterLineHeights(textarea);
                            syncGutterToTextarea(textarea);
                        }
                    });
                });
            }
            if (textarea.dataset.gutterResizeBound !== '1') {
                textarea.dataset.gutterResizeBound = '1';
                const gutterResizeObserver = new ResizeObserver(function() {
                    syncGutterLineHeights(textarea);
                    syncGutterToTextarea(textarea);
                });
                gutterResizeObserver.observe(textarea);
            }
        }

        if (preview && preview.dataset.editorScrollBound !== '1') {
            preview.dataset.editorScrollBound = '1';
            preview.addEventListener('scroll', function() {
                window.onSplitPreviewScroll();
            });
        }

        if (wysiwyg && wysiwyg.dataset.editorScrollBound !== '1') {
            wysiwyg.dataset.editorScrollBound = '1';
            wysiwyg.addEventListener('scroll', function() {
                saveScrollerScroll('wysiwyg-scroll-area', _activeTabId);
                if (window.onViewerScroll) window.onViewerScroll();
            });
        }

        restoreScrollState(_activeTabId);
        syncGutterLineHeights(textarea);
        syncGutterToTextarea(textarea);
        _gutterCurrentLine = 0;
        updateGutterCurrentLine();
        requestAnimationFrame(function() {
            syncGutterLineHeights(textarea);
            syncGutterToTextarea(textarea);
            updateGutterCurrentLine();
        });
        if (window.onViewerScroll) window.onViewerScroll();
    };

    document.addEventListener('selectionchange', function() {
        const ta = document.getElementById('source-markdown-textarea');
        if (!ta || document.activeElement !== ta) return;
        updateGutterCurrentLine();
    });

    window.bindViewerScroll = function(tabId) {
        if (tabId !== undefined && tabId !== null) {
            _activeTabId = String(tabId);
        }
        const viewer = document.getElementById('viewer-scroll-area');
        if (viewer && viewer.dataset.viewerScrollBound !== '1') {
            viewer.dataset.viewerScrollBound = '1';
            viewer.addEventListener('scroll', function() {
                saveScrollerScroll('viewer-scroll-area', _activeTabId);
                if (window.onViewerScroll) window.onViewerScroll();
            });
        }

        restoreScrollState(_activeTabId);
        if (window.onViewerScroll) window.onViewerScroll();
    };
})();

// --- Reading & Curved Outline Progress Engine ---
(function() {
    let _scrollTicking = false;
    let _lastActiveHeadingId = '';
    let _lastScrollerEl = null;
    let _tocPoints = [];
    let _tocLengths = [];
    let _totalTocLength = 0;
    let _activeIdx = 0;
    let _activeFraction = 0;
    let _tocPathTicking = false;

    function resolveScroller() {
        const viewer = document.getElementById('viewer-scroll-area');
        if (viewer) return viewer;
        const splitPreview = document.getElementById('split-preview-scroll-area');
        if (splitPreview) return splitPreview;
        const wysiwyg = document.getElementById('wysiwyg-scroll-area');
        if (wysiwyg) return wysiwyg;
        const textarea = document.getElementById('source-markdown-textarea');
        if (textarea) return textarea;
        return null;
    }

    function isTextareaScroller(el) {
        return el && el.id === 'source-markdown-textarea';
    }

    function computeTocLengthsFromPoints(points) {
        const lengths = [0];
        let total = 0;
        for (let i = 1; i < points.length; i++) {
            const dx = points[i].x - points[i - 1].x;
            const dy = points[i].y - points[i - 1].y;
            total += Math.hypot(dx, dy);
            lengths.push(total);
        }
        return { lengths, total };
    }

    function parseTocMeta(wrapper) {
        const count = parseInt(wrapper.getAttribute('data-toc-count') || '0', 10);
        const rowHeight = parseFloat(wrapper.getAttribute('data-toc-row-height') || '28');
        let levels = [];
        try {
            levels = JSON.parse(wrapper.getAttribute('data-toc-levels') || '[]');
        } catch (e) {
            levels = [];
        }
        return { count, rowHeight, levels };
    }

    function tocIndentPx(level) {
        const indentLevel = Math.min(Math.max(level - 1, 0), 5);
        return indentLevel * 14;
    }

    function tocBulletHalf(level) {
        return level === 1 ? 4 : 3;
    }

    function formulaX(level, pad) {
        return pad + tocIndentPx(level) + tocBulletHalf(level);
    }

    function computeTocPointsFromMeta(wrapper) {
        const { count, rowHeight, levels } = parseTocMeta(wrapper);
        if (count < 2) {
            return [];
        }

        let pad = 8;
        const inner = document.querySelector('#toc-virtual-list > div');
        if (inner) {
            const innerRect = inner.getBoundingClientRect();
            const sample = document.querySelector('#toc-virtual-list .toc-item[data-toc-index] .toc-node-bullet');
            if (sample) {
                const item = sample.closest('.toc-item');
                const idx = parseInt(item.getAttribute('data-toc-index') || '0', 10);
                const level = levels[idx] || 1;
                const b = sample.getBoundingClientRect();
                const x = b.left - innerRect.left + b.width / 2;
                pad = x - tocIndentPx(level) - tocBulletHalf(level);
            }
        }

        const points = [];
        for (let i = 0; i < count; i++) {
            const level = levels[i] || 1;
            points.push({
                x: formulaX(level, pad),
                y: (i + 0.5) * rowHeight,
                id: '',
                item: null,
            });
        }
        return points;
    }

    function refreshTocTreePathImpl() {
        const tocWrapper = document.querySelector('.toc-wrapper');
        const trackPath = document.getElementById('toc-track-path');
        const fillPath = document.getElementById('toc-progress-fill-path');
        const headPip = document.getElementById('toc-progress-head');

        if (!tocWrapper || !trackPath || !fillPath) {
            return;
        }

        setupTocObservers();

        const points = computeTocPointsFromMeta(tocWrapper);
        if (points.length < 2) {
            trackPath.setAttribute('d', '');
            fillPath.setAttribute('d', '');
            if (headPip) headPip.style.opacity = '0';
            _tocPoints = [];
            _tocLengths = [];
            _totalTocLength = 0;
            return;
        }

        _tocPoints = points;

        // Generate continuous smooth Bézier curve connecting every heading node
        let d = `M ${points[0].x.toFixed(1)} ${points[0].y.toFixed(1)}`;
        for (let i = 0; i < points.length - 1; i++) {
            const p1 = points[i];
            const p2 = points[i + 1];
            const dx = p2.x - p1.x;
            const dy = p2.y - p1.y;

            if (Math.abs(dx) < 0.75) {
                // Same indentation level -> straight vertical line
                d += ` L ${p2.x.toFixed(1)} ${p2.y.toFixed(1)}`;
            } else {
                // Indentation step -> beautiful S-curve with vertical tangents
                const cy1 = p1.y + dy * 0.5;
                const cy2 = p1.y + dy * 0.5;
                d += ` C ${p1.x.toFixed(1)} ${cy1.toFixed(1)}, ${p2.x.toFixed(1)} ${cy2.toFixed(1)}, ${p2.x.toFixed(1)} ${p2.y.toFixed(1)}`;
            }
        }

        trackPath.setAttribute('d', d);
        fillPath.setAttribute('d', d);

        const { lengths, total } = computeTocLengthsFromPoints(points);
        let svgLen = total;
        try {
            const measured = fillPath.getTotalLength();
            if (measured > 0) svgLen = measured;
        } catch (e) { /* path not in layout yet */ }
        const scale = total > 0 ? svgLen / total : 1;
        _tocLengths = lengths.map((len) => len * scale);
        _totalTocLength = svgLen;
        fillPath.style.strokeDasharray = `${svgLen.toFixed(1)} ${(svgLen + 40).toFixed(1)}`;

        updateTocFill();
    }

    function scheduleRefreshTocTreePath() {
        if (_tocPathTicking) return;
        _tocPathTicking = true;
        window.requestAnimationFrame(() => {
            _tocPathTicking = false;
            refreshTocTreePathImpl();
        });
    }

    function refreshTocTreePath() {
        scheduleRefreshTocTreePath();
    }

    function updateTocFill() {
        const fillPath = document.getElementById('toc-progress-fill-path');
        const headPip = document.getElementById('toc-progress-head');
        if (!fillPath || _totalTocLength <= 0 || _tocLengths.length === 0) return;

        const k = _activeIdx;
        const fraction = _activeFraction;

        let targetLen = 0;
        if (k >= 0 && k < _tocLengths.length) {
            const L_k = _tocLengths[k];
            const L_next = (k + 1 < _tocLengths.length) ? _tocLengths[k + 1] : _totalTocLength;
            targetLen = L_k + fraction * (L_next - L_k);
        }

        targetLen = Math.min(_totalTocLength, Math.max(0, targetLen));
        const offset = Math.max(0, _totalTocLength - targetLen);
        fillPath.style.strokeDashoffset = `${offset.toFixed(1)}px`;

        if (headPip) {
            if (targetLen > 1 && _totalTocLength > 0) {
                try {
                    const svgLen = fillPath.getTotalLength();
                    const ratio = Math.min(1, Math.max(0, targetLen / _totalTocLength));
                    const pt = fillPath.getPointAtLength(ratio * svgLen);
                    headPip.setAttribute('cx', pt.x.toFixed(1));
                    headPip.setAttribute('cy', pt.y.toFixed(1));
                    headPip.style.opacity = '1';
                } catch (e) {
                    headPip.style.opacity = '0';
                }
            } else {
                headPip.style.opacity = '0';
            }
        }
    }

    function setupTocObservers() {
        const tocWrapper = document.querySelector('.toc-wrapper');
        if (!tocWrapper || tocWrapper.dataset.tocObserved === '1') return;
        tocWrapper.dataset.tocObserved = '1';

        if (window.ResizeObserver) {
            const ro = new ResizeObserver(() => {
                scheduleRefreshTocTreePath();
            });
            ro.observe(tocWrapper);
        }
    }

    function computeActiveHeadingAndFractionFromHtml(scrollArea) {
        const headings = scrollArea.querySelectorAll('.doc-heading, h1[id], h2[id], h3[id], h4[id], h5[id], h6[id]');
        if (!headings || headings.length === 0) return { activeId: null, activeIdx: 0, fraction: 0 };

        const scrollTop = scrollArea.scrollTop;
        const scrollHeight = scrollArea.scrollHeight;
        const clientHeight = scrollArea.clientHeight;
        const offsetThreshold = 110;
        const containerRect = scrollArea.getBoundingClientRect();

        if (scrollTop + clientHeight >= scrollHeight - 15) {
            const lastHeading = headings[headings.length - 1];
            return {
                activeId: lastHeading.id,
                activeIdx: headings.length - 1,
                fraction: 1.0
            };
        }

        let activeIdx = 0;
        let activeId = headings[0].id;
        let headingTops = [];

        for (let i = 0; i < headings.length; i++) {
            const h = headings[i];
            const rect = h.getBoundingClientRect();
            const relativeTop = rect.top - containerRect.top;
            headingTops.push(relativeTop);

            if (relativeTop <= offsetThreshold) {
                activeIdx = i;
                activeId = h.id;
            }
        }

        let fraction = 0;
        if (activeIdx < headings.length - 1) {
            const topCurrent = headingTops[activeIdx];
            const topNext = headingTops[activeIdx + 1];
            const span = topNext - topCurrent;
            if (span > 1) {
                fraction = Math.min(1.0, Math.max(0.0, (offsetThreshold - topCurrent) / span));
            }
        }

        return { activeId, activeIdx, fraction };
    }

    function computeActiveHeadingAndFractionFromTextarea(textarea) {
        const lines = textarea.value.split('\n');
        const slugify = window.slugifyHeading || function(t) { return t.toLowerCase().replace(/[^\w]+/g, '-'); };
        const lineHeight = parseFloat(getComputedStyle(textarea).lineHeight) || 21;
        const visibleLine = Math.floor(textarea.scrollTop / lineHeight);

        let headings = [];
        for (let i = 0; i < lines.length; i++) {
            const m = lines[i].match(/^(#{1,6})\s+(.+)$/);
            if (m) {
                headings.push({ line: i, id: slugify(m[2].trim()) });
            }
        }

        if (headings.length === 0) {
            return { activeId: null, activeIdx: 0, fraction: 0 };
        }

        let activeIdx = 0;
        for (let i = 0; i < headings.length; i++) {
            if (headings[i].line <= visibleLine) {
                activeIdx = i;
            } else {
                break;
            }
        }

        let fraction = 0;
        if (activeIdx < headings.length - 1) {
            const curLine = headings[activeIdx].line;
            const nextLine = headings[activeIdx + 1].line;
            const span = nextLine - curLine;
            if (span > 0) {
                fraction = Math.min(1.0, Math.max(0.0, (visibleLine - curLine) / span));
            }
        }

        return {
            activeId: headings[activeIdx].id,
            activeIdx: activeIdx,
            fraction: fraction
        };
    }

    function applyTocHighlight(info) {
        if (!info || !info.activeId) {
            document.querySelectorAll('#toc-virtual-list .toc-item').forEach((item) => {
                item.classList.remove('active-toc-item', 'passed-toc-item', 'future-toc-item');
            });
            _lastActiveHeadingId = '';
            _activeIdx = 0;
            _activeFraction = 0;
            updateTocFill();
            return;
        }

        const tocWrapper = document.querySelector('.toc-wrapper');
        let resolvedIdx = info.activeIdx;
        if (tocWrapper && info.activeId) {
            try {
                const ids = JSON.parse(tocWrapper.getAttribute('data-toc-ids') || '[]');
                const byId = ids.indexOf(info.activeId);
                if (byId >= 0) resolvedIdx = byId;
            } catch (e) { /* keep viewer index */ }
        }

        _activeIdx = resolvedIdx;
        _activeFraction = info.fraction;

        const tocItems = document.querySelectorAll('#toc-virtual-list .toc-item');
        let activeElem = null;

        tocItems.forEach((item) => {
            const idx = parseInt(item.getAttribute('data-toc-index') || '-1', 10);
            const hId = item.getAttribute('data-heading-id');
            if (hId === info.activeId || idx === resolvedIdx) {
                item.classList.add('active-toc-item');
                item.classList.remove('passed-toc-item', 'future-toc-item');
                activeElem = item;
            } else if (idx >= 0 && idx < resolvedIdx) {
                item.classList.add('passed-toc-item');
                item.classList.remove('active-toc-item', 'future-toc-item');
            } else {
                item.classList.add('future-toc-item');
                item.classList.remove('active-toc-item', 'passed-toc-item');
            }
        });

        if (info.activeId !== _lastActiveHeadingId) {
            _lastActiveHeadingId = info.activeId;
            if (!window._userIsHoveringToc) {
                const tocScroller = document.getElementById('toc-virtual-list');
                if (tocScroller && tocWrapper && resolvedIdx >= 0) {
                    const rowHeight = parseFloat(tocWrapper.getAttribute('data-toc-row-height') || '28');
                    const itemTop = resolvedIdx * rowHeight;
                    const itemBottom = itemTop + rowHeight;
                    const containerTop = tocScroller.scrollTop;
                    const containerBottom = containerTop + tocScroller.clientHeight;

                    if (itemTop < containerTop + 30) {
                        tocScroller.scrollTop = itemTop - 30;
                    } else if (itemBottom > containerBottom - 30) {
                        tocScroller.scrollTop = itemBottom - tocScroller.clientHeight + 30;
                    }
                }
            }
        }

        updateTocFill();
    }

    function updateScrollProgress() {
        _scrollTicking = false;
        try {
            const scrollArea = resolveScroller();
            if (!scrollArea) return;

            if (_lastScrollerEl !== scrollArea) {
                _lastScrollerEl = scrollArea;
                _lastActiveHeadingId = '';
            }

            if (scrollArea.id === 'viewer-scroll-area') {
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
            }

            if (_tocPoints.length === 0) {
                scheduleRefreshTocTreePath();
            }

            let activeInfo;
            if (isTextareaScroller(scrollArea)) {
                activeInfo = computeActiveHeadingAndFractionFromTextarea(scrollArea);
            } else {
                activeInfo = computeActiveHeadingAndFractionFromHtml(scrollArea);
            }

            applyTocHighlight(activeInfo);
        } catch(e) {
            console.error('Progress update error:', e);
        }
    }

    window.refreshTocTreePath = refreshTocTreePath;

    window.onViewerScroll = function() {
        if (window.saveCurrentScroll) window.saveCurrentScroll();
        if (!_scrollTicking) {
            window.requestAnimationFrame(updateScrollProgress);
            _scrollTicking = true;
        }
    };

    window.addEventListener('resize', () => {
        scheduleRefreshTocTreePath();
        if (!_scrollTicking) {
            window.requestAnimationFrame(updateScrollProgress);
            _scrollTicking = true;
        }
    });

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => {
            setTimeout(scheduleRefreshTocTreePath, 50);
        });
    } else {
        setTimeout(scheduleRefreshTocTreePath, 50);
    }
})();
