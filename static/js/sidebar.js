/**
 * SIDEBAR — Click-Only Toggle + Hover Peek
 *
 * - Pinned state persists via server sync and data-pinned attribute.
 * - Hover temporarily expands when collapsed (does NOT change pinned state).
 * - The toggle button (#sidebar-toggle) is the only way to change pinned state.
 */
(function () {
  "use strict";
  const sidebar = document.getElementById("sidebar");
  const toggleBtn = document.getElementById("sidebar-toggle");
  const appShell = document.querySelector(".app-shell");
  if (!sidebar || !toggleBtn) return;

  // Read initial state from the server-rendered data attribute.
  let expanded = sidebar.dataset.pinned === "true";

  // ── State Mutations ────────────────────────────────────────────────────
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

  // Apply initial state immediately on page load.
  if (expanded) {
    expand();
  }

  function syncWithServer(isPinned) {
    fetch("/ui/sidebar/pin", {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: "pinned=" + isPinned,
    }).catch(function () {});
  }

  // ── Hover Expand (peek) ────────────────────────────────────────────────
  // Only peeks when not pinned. Does NOT affect the pinned state.
  sidebar.addEventListener("mouseenter", function () {
    if (!expanded) expand();
  });

  sidebar.addEventListener("mouseleave", function () {
    if (!expanded) collapse();
  });

  // ── Toggle Button ──────────────────────────────────────────────────────
  // The ONLY control that changes the pinned state.
  toggleBtn.addEventListener("click", function () {
    expanded = !expanded;
    sidebar.dataset.pinned = String(expanded);
    if (expanded) {
      expand();
    } else {
      collapse();
    }
    syncWithServer(expanded);
  });
})();
