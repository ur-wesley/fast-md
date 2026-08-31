(function () {
    window.__ctxHref = '';

    document.addEventListener(
        'contextmenu',
        function (e) {
            const target = e.target;
            if (!target || !target.closest) {
                window.__ctxHref = '';
                return;
            }
            const anchor = target.closest('a[href]');
            if (!anchor) {
                window.__ctxHref = '';
                return;
            }
            const href = anchor.getAttribute('href') || '';
            if (href && !href.startsWith('javascript:')) {
                window.__ctxHref = href;
            } else {
                window.__ctxHref = '';
            }
        },
        true
    );
})();
