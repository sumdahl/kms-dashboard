/* Global palette (Cmd/Ctrl+K): open/close + arrow nav. Depends on htmx. */
(function () {
    "use strict";

    var activeIndex = -1;

    function openSearch() {
        var modal = document.getElementById("search-modal");
        var input = document.getElementById("search-input");
        if (!modal || !input) return;
        modal.classList.remove("hidden");
        input.value = "";
        input.focus();
        activeIndex = -1;
        document.body.style.overflow = "hidden";
    }

    function closeSearch() {
        var modal = document.getElementById("search-modal");
        if (!modal) return;
        modal.classList.add("hidden");
        document.body.style.overflow = "";
    }

    window.openSearch = openSearch;
    window.closeSearch = closeSearch;

    window.addEventListener("keydown", function (e) {
        var modal = document.getElementById("search-modal");
        if (!modal) return;

        if (modal.classList.contains("hidden")) {
            if ((e.metaKey || e.ctrlKey) && e.key === "k") {
                e.preventDefault();
                openSearch();
            }
            return;
        }

        var results = modal.querySelectorAll(
            "#search-results a, #search-results .cursor-default"
        );

        if (e.key === "ArrowDown") {
            e.preventDefault();
            activeIndex = Math.min(activeIndex + 1, results.length - 1);
            updateSelection(results);
        } else if (e.key === "ArrowUp") {
            e.preventDefault();
            activeIndex = Math.max(activeIndex - 1, 0);
            updateSelection(results);
        } else if (e.key === "Enter" && activeIndex >= 0) {
            e.preventDefault();
            results[activeIndex].click();
        } else if (e.key === "Escape") {
            closeSearch();
        }
    });

    function updateSelection(results) {
        results.forEach(function (el, i) {
            if (i === activeIndex) {
                el.classList.add("bg-surface-02");
                el.scrollIntoView({ block: "nearest" });
                var arrow = el.querySelector(".lucide-arrow-right");
                if (arrow) arrow.style.opacity = "1";
            } else {
                el.classList.remove("bg-surface-02");
                var arrow2 = el.querySelector(".lucide-arrow-right");
                if (arrow2) arrow2.style.opacity = "0";
            }
        });
    }

    document.addEventListener("htmx:afterOnLoad", function (e) {
        if (e.detail.target && e.detail.target.id === "search-results") {
            activeIndex = -1;
        }
    });
})();
