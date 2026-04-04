/**
 * Shared navbar component.
 * Included in <head> (no defer) so theme is applied synchronously before paint,
 * then the navbar HTML is injected on DOMContentLoaded.
 */
(function () {
    // Apply theme immediately to avoid jitter/flash
    const savedTheme = localStorage.getItem('yappl-theme') || 'dark';
    document.documentElement.setAttribute('data-bs-theme', savedTheme);

    // Inject NavBar when DOM Content has loaded
    document.addEventListener('DOMContentLoaded', function () {
        const path = window.location.pathname;
        const isEditor = path === '/' || path.endsWith('/index.html');
        const isDocs   = path.endsWith('/documentation.html');
        const theme    = document.documentElement.getAttribute('data-bs-theme');

        const navHTML = `
<nav class="navbar border-bottom px-3" id="main-navbar">
    <div class="d-flex align-items-center gap-2 flex-grow-1">
        ${isEditor ? `
        <button class="btn btn-sm btn-outline-secondary d-lg-none" type="button"
                data-bs-toggle="offcanvas" data-bs-target="#sidebar" aria-controls="sidebar"
                title="Show programs">
            <i class="bi bi-layout-sidebar"></i>
        </button>` : ''}
        <a class="navbar-brand mb-0 fw-semibold fs-5 text-decoration-none" href="/">
            <i class="bi bi-braces text-info"></i> YAPPL Playground
        </a>
        <div class="d-flex gap-1 ms-1">
            <a class="btn btn-sm ${isEditor ? 'btn-secondary' : 'btn-outline-secondary'}" href="/">
                <i class="bi bi-code-slash"></i> Editor
            </a>
            <a class="btn btn-sm ${isDocs ? 'btn-secondary' : 'btn-outline-secondary'}" href="/documentation.html">
                <i class="bi bi-book"></i> Docs
            </a>
        </div>
    </div>
    <div class="d-flex align-items-center gap-2">
        <button id="theme-toggle" class="btn btn-sm btn-outline-secondary" title="Toggle light/dark theme">
            <i class="${theme === 'dark' ? 'bi bi-moon-stars-fill' : 'bi bi-sun-fill'}" id="theme-icon"></i>
        </button>
    </div>
</nav>`;

        document.body.insertAdjacentHTML('afterbegin', navHTML);

        // Theme Toggle (class based)
        document.getElementById('theme-toggle').addEventListener('click', function () {
            const current = document.documentElement.getAttribute('data-bs-theme');
            const next = current === 'dark' ? 'light' : 'dark';
            document.documentElement.setAttribute('data-bs-theme', next);
            localStorage.setItem('yappl-theme', next);
            document.getElementById('theme-icon').className =
                next === 'dark' ? 'bi bi-moon-stars-fill' : 'bi bi-sun-fill';
            document.dispatchEvent(new CustomEvent('themechange', { detail: { theme: next } }));
        });
    });
})();
