//! HTMX out-of-band (`hx-swap-oob`) HTML for `#app-notification-container` (global messages).
//!
//! - `GET /ui/global-message?message=…&kind=…` — append a row (`beforeend` OOB).
//! - Row dismiss: `data-app-notification` + X removes the row client-side.

use uuid::Uuid;

fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

fn icon_svg_for_kind(kind: &str) -> &'static str {
    match kind {
        "success" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>"#
        }
        "error" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>"#
        }
        "warning" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3"/><path d="M12 9v4"/><path d="M12 17h.01"/></svg>"#
        }
        _ => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/></svg>"#
        }
    }
}

fn row_html(message: &str, kind: &str) -> String {
    let kind = match kind {
        "success" | "error" | "info" | "warning" => kind,
        _ => "info",
    };
    let id = format!("gm-{}", Uuid::new_v4());
    let escaped = escape_html(message);
    let icon = icon_svg_for_kind(kind);
    let (accent, bg) = match kind {
        "success" => (
            "var(--color-status-green)",
            "var(--color-banner-success-bg)",
        ),
        "error" => ("var(--color-status-red)", "var(--color-banner-danger-bg)"),
        "warning" => ("var(--color-status-yellow)", "var(--color-banner-bg)"),
        _ => ("var(--color-status-blue)", "var(--color-banner-bg)"),
    };

    format!(
        concat!(
            r#"<div id=""#,
            r#"{id}"#,
            r#"" data-app-notification data-auto-dismiss="5000" class="flex items-center justify-between gap-3 px-4 md:px-8 py-2 border-b border-border-subtle min-h-10 shrink-0" style="background:"#,
            r#"{bg}"#,
            r#";" role="alert"><div class="flex items-center gap-3 min-w-0 flex-1"><span class="shrink-0" style="color:"#,
            r#"{accent}"#,
            r#";">{icon}</span><p class="text-xs sm:text-sm font-medium text-text-primary leading-snug wrap-break-word flex-1">"#,
            r#"{escaped}"#,
            r#"</p></div><button type="button" class="relative z-1 shrink-0 flex h-8 w-8 items-center justify-center rounded-sm text-banner-text hover:bg-black/5 dark:hover:bg-white/5 transition-colors" aria-label="Dismiss" onclick="var el=this.closest('[data-app-notification]'); if(el) el.remove();"><svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg></button></div>"#
        ),
        id = id,
        accent = accent,
        bg = bg,
        icon = icon,
        escaped = escaped,
    )
}

fn oob_append_beforeend(row: String) -> String {
    format!(
        r#"<div id="app-notification-container" hx-swap-oob="beforeend">{row}</div>"#
    )
}

pub fn with_success(message: &str) -> String {
    oob_append_beforeend(row_html(message, "success"))
}

pub fn with_error(message: &str) -> String {
    oob_append_beforeend(row_html(message, "error"))
}

pub fn with_warning(message: &str) -> String {
    oob_append_beforeend(row_html(message, "warning"))
}

pub fn with_info(message: &str) -> String {
    oob_append_beforeend(row_html(message, "info"))
}

pub fn from_query_kind(message: &str, kind: Option<&str>) -> String {
    match kind.unwrap_or("info").trim() {
        "success" => with_success(message),
        "error" => with_error(message),
        "warning" => with_warning(message),
        "info" => with_info(message),
        _ => with_info(message),
    }
}
