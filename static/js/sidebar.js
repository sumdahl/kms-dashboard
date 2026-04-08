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
  const mobileToggleBtn = document.getElementById("mobile-sidebar-toggle");
  const overlay = document.getElementById("sidebar-overlay");

  if (!sidebar || !toggleBtn) return;

  // Read initial state from the server-rendered data attribute.
  let expanded = sidebar.dataset.pinned === "true";
  let mobileOpen = false;

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

  // ── Mobile State Mutations ─────────────────────────────────────────────
  function openMobile() {
    mobileOpen = true;
    sidebar.classList.add("mobile-open");
    if (overlay) overlay.classList.add("mobile-open");
    document.body.style.overflow = "hidden"; // prevent scrolling behind
  }

  function closeMobile() {
    mobileOpen = false;
    sidebar.classList.remove("mobile-open");
    if (overlay) overlay.classList.remove("mobile-open");
    document.body.style.overflow = ""; // restore scrolling
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
  // Only peeks when not pinned and not on mobile.
  sidebar.addEventListener("mouseenter", function () {
    if (window.innerWidth <= 768) return;
    if (!expanded) expand();
  });

  sidebar.addEventListener("mouseleave", function () {
    if (window.innerWidth <= 768) return;
    if (!expanded) collapse();
  });

  // ── Toggle Button ──────────────────────────────────────────────────────
  // The ONLY control that changes the pinned state (desktop).
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

  // ── Mobile Toggle Button ───────────────────────────────────────────────
  if (mobileToggleBtn) {
    mobileToggleBtn.addEventListener("click", function () {
      if (mobileOpen) {
        closeMobile();
      } else {
        openMobile();
      }
    });
  }

  // Close mobile sidebar when clicking the overlay
  if (overlay) {
    overlay.addEventListener("click", closeMobile);
  }

  // Close mobile sidebar on resize if window becomes desktop
  window.addEventListener("resize", function() {
    if (window.innerWidth > 768 && mobileOpen) {
      closeMobile();
    }
  });
})();
