/**
 * SIDEBAR — Three-Zone Hover Model + Pin Toggle
 *
 * Zone A: #sidebar-hover-zone  — mouseenter/mouseleave triggers expand/collapse
 * Zone B: implicit gap         — natural dead zone (nothing triggers here)
 * Zone C: #sidebar-toggle      — click-only in footer bar, never triggers hover expand
 *
 * The footer toggle (#sidebar-toggle) is a completely separate DOM element
 * outside the sidebar. Hovering over it cannot trigger Zone A's hover events.
 * This is the key architectural choice that makes the three-zone model work
 * without any complex pointer detection logic.
 */

(function () {
  'use strict';

  const sidebar      = document.getElementById('sidebar');
  const hoverZone    = document.getElementById('sidebar-hover-zone');
  const toggleBtn    = document.getElementById('sidebar-toggle');

  if (!sidebar || !hoverZone || !toggleBtn) return;

  // Read initial pin state from the data attribute set by the server.
  // The server reads a cookie and renders the correct initial state.
  let pinned = sidebar.dataset.pinned === 'true';

  // ── State Mutations ────────────────────────────────────────────────────

  function expand() {
    sidebar.classList.add('is-expanded');
    document.body.classList.add('sidebar-expanded');
  }

  function collapse() {
    sidebar.classList.remove('is-expanded');
    document.body.classList.remove('sidebar-expanded');
  }

  function syncWithServer(isPinned) {
    // Notify the Axum backend to persist the pin state (e.g. in a cookie).
    // hx-boost or manual fetch — either works. Using fetch here to avoid
    // requiring HTMX on this script.
    fetch('/ui/sidebar/pin', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: 'pinned=' + isPinned,
    }).catch(function () {
      // Non-critical — the UI is already in the correct state.
    });
  }

  // ── Zone A: Hover Zone ─────────────────────────────────────────────────
  // Only fires when cursor enters/leaves the nav items area of the sidebar.
  // The footer toggle is outside this element so it is never affected.

  hoverZone.addEventListener('mouseenter', function () {
    if (!pinned) expand();
  });

  hoverZone.addEventListener('mouseleave', function () {
    if (!pinned) collapse();
  });

  // ── Zone C: Toggle Button (footer) ─────────────────────────────────────
  // Click-only. Hovering here does NOT expand the sidebar.

  toggleBtn.addEventListener('click', function () {
    pinned = !pinned;
    sidebar.dataset.pinned = String(pinned);

    if (pinned) {
      expand();
    } else {
      collapse();
    }

    syncWithServer(pinned);
  });

  // ── Accordion: Nav Groups (details/summary) ────────────────────────────
  // Collapse all open groups when sidebar collapses so state is clean
  // when user hovers again.

  sidebar.addEventListener('transitionend', function (e) {
    if (e.propertyName !== 'width') return;
    if (!sidebar.classList.contains('is-expanded')) {
      var groups = sidebar.querySelectorAll('details.sidebar__group[open]');
      groups.forEach(function (g) {
        // Don't remove open from groups that were open before collapse —
        // they should re-open when sidebar expands again.
        // So we only visually hide sub-items via CSS, not remove [open].
      });
    }
  });

})();