
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
