
// --- Synchronized Scroll & View Mode Scroll Preservation Engine ---
(function() {
    let isSyncingFromEditor = false;
    let isSyncingFromPreview = false;
    let _isRestoringScroll = false;
    const _tabScrollRatioMap = new Map();
    let _activeTabId = '0';

    function syncGutterToTextarea(textarea) {
        const inner = document.getElementById('source-line-gutter-inner');
        if (inner && textarea) {
            inner.style.transform = 'translateY(-' + textarea.scrollTop + 'px)';
        }
    }

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

    function saveCurrentScroll(tabId) {
        if (_isRestoringScroll) return;
        const tId = (tabId !== undefined && tabId !== null) ? String(tabId) : resolveActiveTabId();
        const scroller = resolveScroller();
        if (!scroller) return;

        const max = scroller.scrollHeight - scroller.clientHeight;
        if (max > 0) {
            const ratio = Math.max(0, Math.min(1, scroller.scrollTop / max));
            _tabScrollRatioMap.set(tId, ratio);
        } else {
            _tabScrollRatioMap.set(tId, 0);
        }
        _activeTabId = tId;
    }

    window.saveCurrentScroll = saveCurrentScroll;

    function restoreScrollState(tabId) {
        if (tabId !== undefined && tabId !== null) {
            _activeTabId = String(tabId);
        }
        const tId = resolveActiveTabId();
        const ratio = _tabScrollRatioMap.has(tId) ? _tabScrollRatioMap.get(tId) : 0;
        if (ratio === undefined || ratio === null) return;

        function applyScroll() {
            _isRestoringScroll = true;
            const viewer = document.getElementById('viewer-scroll-area');
            const splitPreview = document.getElementById('split-preview-scroll-area');
            const wysiwyg = document.getElementById('wysiwyg-scroll-area');
            const textarea = document.getElementById('source-markdown-textarea');

            if (viewer) {
                const max = viewer.scrollHeight - viewer.clientHeight;
                if (max > 0) {
                    viewer.scrollTop = ratio * max;
                }
            }
            if (splitPreview) {
                const max = splitPreview.scrollHeight - splitPreview.clientHeight;
                if (max > 0) {
                    splitPreview.scrollTop = ratio * max;
                }
            }
            if (wysiwyg) {
                const max = wysiwyg.scrollHeight - wysiwyg.clientHeight;
                if (max > 0) {
                    wysiwyg.scrollTop = ratio * max;
                }
            }
            if (textarea) {
                const max = textarea.scrollHeight - textarea.clientHeight;
                if (max > 0) {
                    textarea.scrollTop = ratio * max;
                }
                syncGutterToTextarea(textarea);
            }

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

    window.onEditorSourceScroll = function() {
        const textarea = document.getElementById('source-markdown-textarea');
        const preview = document.getElementById('split-preview-scroll-area');

        syncGutterToTextarea(textarea);

        if (isSyncingFromPreview) {
            return;
        }

        if (textarea && preview) {
            const editorMax = textarea.scrollHeight - textarea.clientHeight;
            const previewMax = preview.scrollHeight - preview.clientHeight;

            if (editorMax > 0 && previewMax > 0) {
                isSyncingFromEditor = true;
                preview.scrollTop = (textarea.scrollTop / editorMax) * previewMax;
                isSyncingFromEditor = false;
            }
        }

        saveCurrentScroll();
        if (window.onViewerScroll) window.onViewerScroll();
    };

    window.onSplitPreviewScroll = function() {
        if (isSyncingFromEditor) {
            return;
        }

        const textarea = document.getElementById('source-markdown-textarea');
        const preview = document.getElementById('split-preview-scroll-area');

        if (textarea && preview) {
            const editorMax = textarea.scrollHeight - textarea.clientHeight;
            const previewMax = preview.scrollHeight - preview.clientHeight;

            if (editorMax > 0 && previewMax > 0) {
                isSyncingFromPreview = true;
                textarea.scrollTop = (preview.scrollTop / previewMax) * editorMax;
                syncGutterToTextarea(textarea);
                isSyncingFromPreview = false;
            }
        }

        saveCurrentScroll();
        if (window.onViewerScroll) window.onViewerScroll();
    };

    window.bindEditorScroll = function(tabId) {
        if (tabId !== undefined && tabId !== null) {
            _activeTabId = String(tabId);
        }
        const textarea = document.getElementById('source-markdown-textarea');
        const preview = document.getElementById('split-preview-scroll-area');
        const wysiwyg = document.getElementById('wysiwyg-scroll-area');

        if (textarea && textarea.dataset.editorScrollBound !== '1') {
            textarea.dataset.editorScrollBound = '1';
            textarea.addEventListener('scroll', function() {
                saveCurrentScroll(_activeTabId);
                window.onEditorSourceScroll();
            });
        }

        if (preview && preview.dataset.editorScrollBound !== '1') {
            preview.dataset.editorScrollBound = '1';
            preview.addEventListener('scroll', function() {
                saveCurrentScroll(_activeTabId);
                window.onSplitPreviewScroll();
            });
        }

        if (wysiwyg && wysiwyg.dataset.editorScrollBound !== '1') {
            wysiwyg.dataset.editorScrollBound = '1';
            wysiwyg.addEventListener('scroll', function() {
                saveCurrentScroll(_activeTabId);
                if (window.onViewerScroll) window.onViewerScroll();
            });
        }

        restoreScrollState(_activeTabId);
        syncGutterToTextarea(textarea);
        if (window.onViewerScroll) window.onViewerScroll();
    };

    window.bindViewerScroll = function(tabId) {
        if (tabId !== undefined && tabId !== null) {
            _activeTabId = String(tabId);
        }
        const viewer = document.getElementById('viewer-scroll-area');
        if (viewer && viewer.dataset.viewerScrollBound !== '1') {
            viewer.dataset.viewerScrollBound = '1';
            viewer.addEventListener('scroll', function() {
                saveCurrentScroll(_activeTabId);
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

    function refreshTocTreePath() {
        const tocContainer = document.querySelector('.sidebar-toc-container');
        const tocWrapper = document.querySelector('.toc-wrapper');
        const trackPath = document.getElementById('toc-track-path');
        const fillPath = document.getElementById('toc-progress-fill-path');
        const headPip = document.getElementById('toc-progress-head');

        if (!tocContainer || !tocWrapper || !trackPath || !fillPath) {
            return;
        }

        setupTocObservers();

        const items = tocContainer.querySelectorAll('.toc-item');
        if (items.length < 2) {
            trackPath.setAttribute('d', '');
            fillPath.setAttribute('d', '');
            if (headPip) headPip.style.opacity = '0';
            _tocPoints = [];
            _tocLengths = [];
            _totalTocLength = 0;
            return;
        }

        const wrapperRect = tocWrapper.getBoundingClientRect();
        const points = [];

        items.forEach((item) => {
            const bullet = item.querySelector('.toc-node-bullet');
            const hId = item.getAttribute('data-heading-id') || '';
            if (bullet) {
                const bRect = bullet.getBoundingClientRect();
                const x = bRect.left - wrapperRect.left + bRect.width / 2;
                const y = bRect.top - wrapperRect.top + bRect.height / 2;
                points.push({ x, y, id: hId, item });
            }
        });

        if (points.length < 2) {
            trackPath.setAttribute('d', '');
            fillPath.setAttribute('d', '');
            if (headPip) headPip.style.opacity = '0';
            _tocPoints = points;
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

        try {
            const totalLen = fillPath.getTotalLength();
            _totalTocLength = totalLen;

            _tocLengths = [0];
            for (let i = 1; i < points.length - 1; i++) {
                const targetY = points[i].y;
                let low = 0;
                let high = totalLen;
                for (let iter = 0; iter < 18; iter++) {
                    const mid = (low + high) / 2;
                    const pt = fillPath.getPointAtLength(mid);
                    if (pt.y < targetY) {
                        low = mid;
                    } else {
                        high = mid;
                    }
                }
                _tocLengths.push((low + high) / 2);
            }
            if (points.length >= 2) {
                _tocLengths.push(totalLen);
            }

            fillPath.style.strokeDasharray = `${totalLen.toFixed(1)} ${(totalLen + 40).toFixed(1)}`;
        } catch (e) {
            console.error('Error computing TOC path lengths:', e);
        }

        updateTocFill();
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
            if (targetLen > 1) {
                try {
                    const pt = fillPath.getPointAtLength(targetLen);
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
        const tocContainer = document.querySelector('.sidebar-toc-container');
        if (!tocContainer || tocContainer.dataset.tocObserved === '1') return;
        tocContainer.dataset.tocObserved = '1';

        if (window.ResizeObserver) {
            const ro = new ResizeObserver(() => {
                refreshTocTreePath();
            });
            ro.observe(tocContainer);
            const tocWrapper = document.querySelector('.toc-wrapper');
            if (tocWrapper) ro.observe(tocWrapper);
        }

        if (window.MutationObserver) {
            const mo = new MutationObserver(() => {
                refreshTocTreePath();
            });
            mo.observe(tocContainer, { childList: true, subtree: true });
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
            document.querySelectorAll('.sidebar-toc-container .toc-item').forEach((item) => {
                item.classList.remove('active-toc-item', 'passed-toc-item', 'future-toc-item');
            });
            _lastActiveHeadingId = '';
            _activeIdx = 0;
            _activeFraction = 0;
            updateTocFill();
            return;
        }

        _activeIdx = info.activeIdx;
        _activeFraction = info.fraction;

        const tocItems = document.querySelectorAll('.sidebar-toc-container .toc-item');
        let activeElem = null;

        tocItems.forEach((item, idx) => {
            const hId = item.getAttribute('data-heading-id');
            if (hId === info.activeId || idx === info.activeIdx) {
                item.classList.add('active-toc-item');
                item.classList.remove('passed-toc-item', 'future-toc-item');
                activeElem = item;
            } else if (idx < info.activeIdx) {
                item.classList.add('passed-toc-item');
                item.classList.remove('active-toc-item', 'future-toc-item');
            } else {
                item.classList.add('future-toc-item');
                item.classList.remove('active-toc-item', 'passed-toc-item');
            }
        });

        if (info.activeId !== _lastActiveHeadingId) {
            _lastActiveHeadingId = info.activeId;
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
                refreshTocTreePath();
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
        refreshTocTreePath();
        if (!_scrollTicking) {
            window.requestAnimationFrame(updateScrollProgress);
            _scrollTicking = true;
        }
    });

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => {
            setTimeout(refreshTocTreePath, 50);
        });
    } else {
        setTimeout(refreshTocTreePath, 50);
    }
})();
