/**
 * SIDEBAR — Click-Only Toggle
 *
 * The sidebar defaults to expanded (server sends pinned=true).
 * It does NOT react to mouse hover at all.
 * The ONLY way to compress/expand it is via the toggle button (#sidebar-toggle).
 */

(function () {
  'use strict';

  const sidebar = document.getElementById('sidebar');
  const toggleBtn = document.getElementById('sidebar-toggle');

  if (!sidebar || !toggleBtn) return;

  // Read initial state from the server-rendered data attribute.
  let expanded = sidebar.dataset.pinned === 'true';

  // ── State Mutations ────────────────────────────────────────────────────

  function expand() {
    sidebar.classList.add('is-expanded');
    document.body.classList.add('sidebar-expanded');
  }

  function collapse() {
    sidebar.classList.remove('is-expanded');
    document.body.classList.remove('sidebar-expanded');
  }

  // Apply initial state immediately on page load
  if (expanded) {
    expand();
  }

  function syncWithServer(isPinned) {
    fetch('/ui/sidebar/pin', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: 'pinned=' + isPinned,
    }).catch(function () {});
  }

  // ── Toggle Button ──────────────────────────────────────────────────────
  // The ONLY control. Click to expand or compress.

  toggleBtn.addEventListener('click', function () {
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