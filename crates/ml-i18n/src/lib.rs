//! Localization and Fluent string management for Moonlight (en-US).

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use std::cell::RefCell;
use unic_langid::LanguageIdentifier;

/// Embedded Fluent translation resources for en-US.
const EN_US_FTL: &str = r#"
toolbar-back = Back
toolbar-forward = Forward
toolbar-reload = Reload
omnibox-placeholder = Search DuckDuckGo or type a URL
tab-new = New Tab
tab-close = Close tab
tab-default-title = New Tab
newtab-title = New Tab
newtab-search-placeholder = Search DuckDuckGo or type a URL
newtab-shortcut-github = GitHub
newtab-shortcut-docs = docs.rs
newtab-shortcut-so = Stack Overflow
newtab-shortcut-mdn = MDN Web Docs
newtab-shortcut-add = Add shortcut
newtab-stat-trackers = Trackers blocked
newtab-stat-ram = RAM used
newtab-stat-loadtime = Load time saved
error-title = Unable to load page
error-offline = Network offline or connection timed out
error-cert = Security certificate invalid
error-crash = Tab crashed unexpectedly
"#;

thread_local! {
    static BUNDLE: RefCell<FluentBundle<FluentResource>> = RefCell::new({
        let lang: LanguageIdentifier = "en-US".parse().expect("valid language identifier");
        let mut bundle = FluentBundle::new(vec![lang]);
        let resource = FluentResource::try_new(EN_US_FTL.to_string())
            .expect("embedded en-US FTL must be valid");
        bundle
            .add_resource(resource)
            .expect("failed to add en-US resource to bundle");
        bundle
    });
}

/// Look up a localized string by key without arguments.
pub fn text(id: &str) -> String {
    text_with_args(id, None)
}

/// Look up a localized string by key with optional arguments.
pub fn text_with_args(id: &str, args: Option<&FluentArgs>) -> String {
    BUNDLE.with(|bundle_cell| {
        let bundle = bundle_cell.borrow();
        let pattern = match bundle.get_message(id).and_then(|msg| msg.value()) {
            Some(pattern) => pattern,
            None => return id.to_string(),
        };

        let mut errors = vec![];
        let value = bundle.format_pattern(pattern, args, &mut errors);
        value.to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_existing_keys() {
        assert_eq!(text("toolbar-back"), "Back");
        assert_eq!(text("toolbar-forward"), "Forward");
        assert_eq!(text("toolbar-reload"), "Reload");
        assert_eq!(
            text("omnibox-placeholder"),
            "Search DuckDuckGo or type a URL"
        );
    }

    #[test]
    fn test_fallback_on_missing_key() {
        assert_eq!(text("unknown-key"), "unknown-key");
    }
}
