/* Lucide + theme toggle + SPA nav — run after lucide.min.js (same defer batch). */
(function () {
    function initLucideIcons() {
        if (window.lucide) window.lucide.createIcons();
    }

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", initLucideIcons);
    } else {
        initLucideIcons();
    }

    document.addEventListener("htmx:afterSwap", initLucideIcons);

    /* ── SPA nav active state ─────────────────────────────────────── */

    function updateNavActive(path) {
        document.querySelectorAll("[data-nav]").forEach(function (link) {
            var nav = link.dataset.nav;
            var isActive = false;

            if (nav === "home") {
                isActive = path === "/" || path === "";
            } else {
                isActive =
                    path === "/" + nav || path.startsWith("/" + nav + "/");
            }

            var icon = link.querySelector("[data-lucide]");

            if (isActive) {
                link.classList.add("bg-surface-active", "text-text-primary", "font-semibold");
                link.classList.remove("text-text-secondary", "hover:bg-surface-hover", "hover:text-text-primary");
                link.setAttribute("aria-current", "page");
                if (icon) {
                    icon.classList.add("text-icon-secondary");
                    icon.classList.remove("text-icon-muted");
                }
            } else {
                link.classList.remove("bg-surface-active", "text-text-primary", "font-semibold");
                link.classList.add("text-text-secondary", "hover:bg-surface-hover", "hover:text-text-primary");
                link.removeAttribute("aria-current");
                if (icon) {
                    icon.classList.remove("text-icon-secondary");
                    icon.classList.add("text-icon-muted");
                }
            }
        });
    }

    document.addEventListener("htmx:afterSettle", function (e) {
        if (e.detail.target && e.detail.target.id === "dashboard-outlet") {
            updateNavActive(window.location.pathname);
            /* scroll outlet's scroll container back to top */
            var main = document.querySelector("main.flex-1");
            if (main) main.scrollTop = 0;
        }
    });

    var html = document.documentElement;

    function syncThemeUI() {
        var isDark = html.classList.contains("dark");
        var sun = document.getElementById("icon-sun");
        var moon = document.getElementById("icon-moon");
        var textEl = document.getElementById("theme-text");
        if (sun) sun.classList.toggle("hidden", !isDark);
        if (moon) moon.classList.toggle("hidden", isDark);
        if (textEl)
            textEl.textContent = isDark ? "Light mode" : "Dark mode";
    }

    syncThemeUI();

    document.addEventListener("click", function (e) {
        var toggle = e.target.closest("#theme-toggle");
        if (!toggle) return;

        html.classList.add("no-transition");
        var isDark = html.classList.toggle("dark");
        localStorage.setItem("theme", isDark ? "dark" : "light");
        syncThemeUI();
        window.getComputedStyle(html).opacity;
        html.classList.remove("no-transition");
    });

    document.addEventListener("htmx:afterSwap", function (e) {
        if (e.detail.target && e.detail.target.id === "account-dropdown") {
            syncThemeUI();
        }
    });
})();
