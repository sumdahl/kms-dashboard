/**
 * SIDEBAR — Visual Hover Peek only.
 *
 * - Persistent pinned state is now handled by HTMX + Cookies.
 * - This script only handles the immediate visual expand/collapse on hover
 *   when the sidebar is not pinned.
 */
(function () {
  "use strict";
  const sidebar = document.getElementById("sidebar");
  const appShell = document.querySelector(".app-shell");

  if (!sidebar) return;

  function expand() {
    sidebar.classList.add("is-expanded");
    document.body.classList.add("sidebar-expanded");
    if (appShell) appShell.classList.add("sidebar-expanded");
  }

  function collapse() {
    sidebar.classList.remove("is-expanded");
    document.body.classList.remove("sidebar-expanded");
    if (appShell) appShell.classList.remove("sidebar-expanded");
  }

  // Hover Expand (peek)
  sidebar.addEventListener("mouseenter", function () {
    if (window.innerWidth <= 768) return;
    if (sidebar.dataset.pinned === "false") expand();
  });

  sidebar.addEventListener("mouseleave", function () {
    if (window.innerWidth <= 768) return;
    if (sidebar.dataset.pinned === "false") collapse();
  });

  // HTMX Re-initialization
  // Since HTMX replaces the sidebar, we might need to re-bind listeners
  // or use event delegation. Let's use event delegation on document.
})();
