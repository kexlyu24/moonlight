//! Internal moonlight:// pages assets, routing, and HTML templates.
//!
//! Provides handlers and renderers for internal browser pages:
//! - New Tab (`moonlight://newtab` and `about:newtab`)
//! - Settings (`moonlight://settings`)
//! - Extensions (`moonlight://extensions`)
//! - History (`moonlight://history`)
//! - Bookmarks (`moonlight://bookmarks`)
//! - Downloads (`moonlight://downloads`)
//! - Error pages (`moonlight://error`)

use url::Url;

/// Classification of internal Moonlight pages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InternalPage {
    NewTab,
    Settings,
    Extensions,
    History,
    Bookmarks,
    Downloads,
    Error {
        title: String,
        description: String,
        target_url: Option<String>,
    },
}

/// HTTP-style response for an internal page request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternalPageResponse {
    pub status: u16,
    pub content_type: &'static str,
    pub body: String,
}

/// Check if a URL targets an internal browser page.
pub fn is_internal_url(url: &Url) -> bool {
    parse_internal_url(url).is_some()
}

/// Parse a URL into its corresponding `InternalPage`, if valid.
pub fn parse_internal_url(url: &Url) -> Option<InternalPage> {
    match url.scheme() {
        "moonlight" => {
            let host_or_path = match url.host_str() {
                Some(host) => host,
                None => url.path().trim_start_matches('/'),
            };

            match host_or_path {
                "newtab" => Some(InternalPage::NewTab),
                "settings" => Some(InternalPage::Settings),
                "extensions" => Some(InternalPage::Extensions),
                "history" => Some(InternalPage::History),
                "bookmarks" => Some(InternalPage::Bookmarks),
                "downloads" => Some(InternalPage::Downloads),
                "error" => {
                    let desc = url
                        .query_pairs()
                        .find(|(k, _)| k == "desc")
                        .map(|(_, v)| v.into_owned())
                        .unwrap_or_else(|| ml_i18n::text("error-offline"));
                    let target = url
                        .query_pairs()
                        .find(|(k, _)| k == "url")
                        .map(|(_, v)| v.into_owned());
                    Some(InternalPage::Error {
                        title: ml_i18n::text("error-title"),
                        description: desc,
                        target_url: target,
                    })
                }
                _ => None,
            }
        }
        "about" => {
            if url.path() == "newtab" {
                Some(InternalPage::NewTab)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Render the HTML content for an internal page.
pub fn render_internal_page(page: &InternalPage) -> InternalPageResponse {
    let body = match page {
        InternalPage::NewTab => render_newtab_html(),
        InternalPage::Settings => {
            render_placeholder_page("Settings", "Browser preferences and configuration.")
        }
        InternalPage::Extensions => {
            render_placeholder_page("Extensions", "Manage Manifest V3 Chrome extensions.")
        }
        InternalPage::History => {
            render_placeholder_page("History", "Browsing history and recent sessions.")
        }
        InternalPage::Bookmarks => {
            render_placeholder_page("Bookmarks", "Saved bookmarks and bookmarks bar.")
        }
        InternalPage::Downloads => {
            render_placeholder_page("Downloads", "Active and completed downloads.")
        }
        InternalPage::Error {
            title,
            description,
            target_url,
        } => render_error_html(title, description, target_url.as_deref()),
    };

    InternalPageResponse {
        status: 200,
        content_type: "text/html; charset=utf-8",
        body,
    }
}

/// Convert an internal page response to a `data:text/html;charset=utf-8,...` URL for engine loading.
pub fn to_data_url(response: &InternalPageResponse) -> Option<Url> {
    let encoded: String = url::form_urlencoded::byte_serialize(response.body.as_bytes()).collect();
    let mime = response
        .content_type
        .split(';')
        .next()
        .unwrap_or("text/html");
    Url::parse(&format!("data:{mime};charset=utf-8,{encoded}")).ok()
}

const NEWTAB_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8"><title>New Tab</title>
  <style>
    :root { --bg: #0E1117; --s1: #161A23; --s2: #262C3D; --si: #2C3343; --st: #262B38; --b: #2A3142; --fg: #E6E9F2; --muted: #94A3B8; --moon: #C7A6FF; --accent: #8FB3FF; }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body { background: var(--bg); color: var(--fg); font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; min-height: 100vh; display: flex; flex-direction: column; align-items: center; overflow-x: hidden; }
    a { color: inherit; text-decoration: none; }
    .glow { position: absolute; top: -100px; width: 600px; height: 300px; background: radial-gradient(circle, #252E47 0%, rgba(14,17,23,0) 70%); pointer-events: none; }
    .wrap { position: relative; width: 100%; max-width: 680px; padding: 80px 20px 40px; display: flex; flex-direction: column; align-items: center; }
    .brand { display: flex; align-items: center; gap: 12px; margin-bottom: 32px; font-size: 26px; font-weight: 600; }
    .brand svg { width: 36px; height: 36px; fill: var(--moon); }
    .search-box { width: 100%; background: var(--si); border: 1px solid var(--b); border-radius: 9999px; padding: 12px 20px; display: flex; align-items: center; gap: 12px; }
    .search-box:focus-within { border-color: var(--accent); box-shadow: 0 0 0 2px rgba(143,179,255,0.2); }
    .search-box input { width: 100%; background: transparent; border: none; outline: none; color: var(--fg); font-size: 15px; }
    .grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px; width: 100%; margin-top: 40px; }
    .card { background: var(--st); border: 1px solid var(--b); border-radius: 10px; padding: 16px; display: flex; flex-direction: column; align-items: center; gap: 10px; font-size: 12px; font-weight: 500; }
    .card:hover { transform: translateY(-2px); border-color: var(--accent); }
    .stats { margin-top: 48px; width: 100%; background: var(--s1); border: 1px solid var(--b); border-radius: 12px; padding: 16px 20px; display: flex; justify-content: space-around; text-align: center; }
    .stat-val { font-size: 18px; font-weight: 700; color: var(--accent); }
    .stat-lbl { font-size: 11px; color: var(--muted); margin-top: 2px; text-transform: uppercase; letter-spacing: 0.05em; }
  </style>
</head>
<body>
  <div class="glow"></div>
  <div class="wrap">
    <div class="brand"><svg viewBox="0 0 24 24"><path d="M12 2A10 10 0 1 0 22 12A10 10 0 0 1 12 2Z"/></svg><span>Moonlight</span></div>
    <form class="search-box" action="https://duckduckgo.com/" method="GET">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#94A3B8" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
      <input type="text" name="q" placeholder="__SEARCH_PLACEHOLDER__" autofocus autocomplete="off" />
    </form>
    <div class="grid">
      <a class="card" href="https://github.com/"><svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"/></svg><span>__GH_LABEL__</span></a>
      <a class="card" href="https://docs.rs/"><svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1-2.5-2.5Z"/><path d="M6 6h10"/><path d="M6 10h10"/></svg><span>__DOCS_LABEL__</span></a>
      <a class="card" href="https://stackoverflow.com/"><svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><path d="M18.986 21.865v-6.408h2.134V24H3v-8.543h2.134v6.408h13.852zM6.111 19.731h11.72v-2.134H6.111v2.134zm.6-4.606l11.458 2.443.498-2.077-11.458-2.443-.498 2.077zm1.884-4.832l10.378 5.626.996-1.89-10.378-5.626-.996 1.89zm3.565-4.993l8.47 8.167 1.494-1.55-8.47-8.167-1.494 1.55zM15.426 1l-1.89 1.002 6.012 10.155 1.89-1.002L15.426 1z"/></svg><span>__SO_LABEL__</span></a>
      <a class="card" href="https://developer.mozilla.org/"><svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m18 16 4-4-4-4"/><path d="m6 8-4 4 4 4"/><path d="m14.5 4-5 16"/></svg><span>__MDN_LABEL__</span></a>
    </div>
    <div class="stats">
      <div><div class="stat-val">0</div><div class="stat-lbl">__STAT_TRACKERS__</div></div>
      <div><div class="stat-val">34 MB</div><div class="stat-lbl">__STAT_RAM__</div></div>
      <div><div class="stat-val">0.4s</div><div class="stat-lbl">__STAT_LOADTIME__</div></div>
    </div>
  </div>
</body>
</html>
"##;

fn render_newtab_html() -> String {
    NEWTAB_HTML
        .replace(
            "__SEARCH_PLACEHOLDER__",
            &ml_i18n::text("newtab-search-placeholder"),
        )
        .replace("__GH_LABEL__", &ml_i18n::text("newtab-shortcut-github"))
        .replace("__DOCS_LABEL__", &ml_i18n::text("newtab-shortcut-docs"))
        .replace("__SO_LABEL__", &ml_i18n::text("newtab-shortcut-so"))
        .replace("__MDN_LABEL__", &ml_i18n::text("newtab-shortcut-mdn"))
        .replace("__STAT_TRACKERS__", &ml_i18n::text("newtab-stat-trackers"))
        .replace("__STAT_RAM__", &ml_i18n::text("newtab-stat-ram"))
        .replace("__STAT_LOADTIME__", &ml_i18n::text("newtab-stat-loadtime"))
}

fn render_placeholder_page(title: &str, description: &str) -> String {
    format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>{title} - Moonlight</title><style>body {{ background: #0E1117; color: #E6E9F2; font-family: sans-serif; padding: 40px; }} .card {{ background: #161A23; border: 1px solid #2A3142; border-radius: 8px; padding: 24px; max-width: 680px; margin: 0 auto; }} h1 {{ font-size: 22px; margin-bottom: 8px; }} p {{ color: #94A3B8; font-size: 14px; }}</style></head><body><div class="card"><h1>{title}</h1><p>{description}</p></div></body></html>"#
    )
}

fn render_error_html(title: &str, description: &str, target_url: Option<&str>) -> String {
    let url_display = target_url.map(|u| format!(r#"<p style="margin-top:10px;color:#94A3B8;font-size:13px;">Target: <code>{u}</code></p>"#)).unwrap_or_default();
    format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>{title} - Moonlight</title><style>body {{ background: #0E1117; color: #E6E9F2; font-family: sans-serif; display: flex; justify-content: center; align-items: center; min-height: 100vh; margin: 0; }} .card {{ background: #161A23; border: 1px solid #2A3142; border-radius: 12px; padding: 32px; max-width: 500px; text-align: center; }} h1 {{ font-size: 20px; margin-bottom: 12px; }} p {{ color: #94A3B8; font-size: 14px; }} .btn {{ display: inline-block; margin-top: 20px; padding: 8px 16px; background: #262C3D; border: 1px solid #2A3142; border-radius: 6px; color: #E6E9F2; text-decoration: none; font-size: 13px; }}</style></head><body><div class="card"><h1>{title}</h1><p>{description}</p>{url_display}<a class="btn" href="javascript:location.reload()">Reload Page</a></div></body></html>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_internal_urls() {
        assert_eq!(
            parse_internal_url(&Url::parse("moonlight://newtab").unwrap()),
            Some(InternalPage::NewTab)
        );
        assert_eq!(
            parse_internal_url(&Url::parse("about:newtab").unwrap()),
            Some(InternalPage::NewTab)
        );
        assert_eq!(
            parse_internal_url(&Url::parse("moonlight://settings").unwrap()),
            Some(InternalPage::Settings)
        );
        assert_eq!(
            parse_internal_url(&Url::parse("moonlight://extensions").unwrap()),
            Some(InternalPage::Extensions)
        );
        let error = Url::parse("moonlight://error?desc=Offline&url=https://example.com").unwrap();
        assert!(matches!(
            parse_internal_url(&error),
            Some(InternalPage::Error { .. })
        ));
        assert_eq!(
            parse_internal_url(&Url::parse("https://duckduckgo.com/").unwrap()),
            None
        );
    }

    #[test]
    fn test_render_newtab_contains_fluent_strings() {
        let resp = render_internal_page(&InternalPage::NewTab);
        assert_eq!(resp.status, 200);
        assert!(resp.body.contains("Moonlight"));
        assert!(resp.body.contains("Search DuckDuckGo or type a URL"));
        assert!(resp.body.contains("GitHub"));
        assert!(resp.body.contains("docs.rs"));
        assert!(resp.body.contains("34 MB"));
    }

    #[test]
    fn test_to_data_url_generation() {
        let resp = render_internal_page(&InternalPage::NewTab);
        let data_url = to_data_url(&resp).expect("must generate valid data url");
        assert_eq!(data_url.scheme(), "data");
        assert!(data_url
            .to_string()
            .starts_with("data:text/html;charset=utf-8,"));
    }
}
