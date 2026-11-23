//! Tab and multi-tab state management for Moonlight.

use url::Url;

/// Unique identifier for an open browser tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TabId(pub u64);

/// State representation of a single browser tab.
#[derive(Debug, Clone, PartialEq)]
pub struct Tab {
    pub id: TabId,
    pub title: String,
    pub url: Url,
    pub is_loading: bool,
}

impl Tab {
    pub fn new(id: TabId, url: Url) -> Self {
        let title = Self::derive_title_from_url(&url);
        Self {
            id,
            title,
            url,
            is_loading: false,
        }
    }

    pub fn derive_title_from_url(url: &Url) -> String {
        match url.scheme() {
            "moonlight" => {
                let page = url.host_str().unwrap_or(url.path().trim_start_matches('/'));
                match page {
                    "newtab" => ml_i18n::text("newtab-title"),
                    "settings" => "Settings".to_string(),
                    "extensions" => "Extensions".to_string(),
                    "history" => "History".to_string(),
                    "bookmarks" => "Bookmarks".to_string(),
                    "downloads" => "Downloads".to_string(),
                    other => format!("Moonlight: {other}"),
                }
            }
            "about" => {
                if url.path() == "newtab" {
                    ml_i18n::text("newtab-title")
                } else {
                    ml_i18n::text("tab-default-title")
                }
            }
            _ => match url.host_str() {
                Some(host) => host.to_string(),
                None => ml_i18n::text("tab-default-title"),
            },
        }
    }

    pub fn display_title(&self, max_chars: usize) -> String {
        let trimmed = self.title.trim();
        let display = if trimmed.is_empty() {
            Self::derive_title_from_url(&self.url)
        } else {
            trimmed.to_string()
        };
        if display.chars().count() > max_chars && max_chars > 3 {
            let truncated: String = display.chars().take(max_chars - 3).collect();
            format!("{truncated}...")
        } else {
            display
        }
    }
}

/// Result of closing a tab in the `TabManager`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloseTabResult {
    Closed { new_active: Option<TabId> },
    NoTabsRemaining,
    NotFound,
}

/// Manager maintaining open tabs and active selection.
#[derive(Debug)]
pub struct TabManager {
    tabs: Vec<Tab>,
    active_index: usize,
    next_id: u64,
}

impl TabManager {
    pub fn new(initial_url: Url) -> Self {
        Self {
            tabs: vec![Tab::new(TabId(1), initial_url)],
            active_index: 0,
            next_id: 2,
        }
    }

    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }
    pub fn tabs_mut(&mut self) -> &mut [Tab] {
        &mut self.tabs
    }
    pub fn len(&self) -> usize {
        self.tabs.len()
    }
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }
    pub fn active_index(&self) -> usize {
        self.active_index
    }
    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active_index)
    }
    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active_index)
    }

    pub fn open_tab(&mut self, url: Url) -> TabId {
        let id = TabId(self.next_id);
        self.next_id += 1;
        self.tabs.push(Tab::new(id, url));
        self.active_index = self.tabs.len() - 1;
        id
    }

    pub fn select_tab(&mut self, id: TabId) -> bool {
        if let Some(pos) = self.tabs.iter().position(|t| t.id == id) {
            self.active_index = pos;
            true
        } else {
            false
        }
    }

    pub fn select_index(&mut self, index: usize) -> bool {
        if index < self.tabs.len() {
            self.active_index = index;
            true
        } else {
            false
        }
    }

    pub fn close_tab(&mut self, id: TabId) -> CloseTabResult {
        let pos = match self.tabs.iter().position(|t| t.id == id) {
            Some(p) => p,
            None => return CloseTabResult::NotFound,
        };
        self.tabs.remove(pos);
        if self.tabs.is_empty() {
            self.active_index = 0;
            return CloseTabResult::NoTabsRemaining;
        }
        if pos < self.active_index {
            self.active_index -= 1;
        } else if self.active_index >= self.tabs.len() {
            self.active_index = self.tabs.len() - 1;
        }
        CloseTabResult::Closed {
            new_active: self.active_tab().map(|t| t.id),
        }
    }

    pub fn move_tab(&mut self, from_index: usize, to_index: usize) -> bool {
        if from_index >= self.tabs.len() || to_index >= self.tabs.len() {
            return false;
        }
        if from_index == to_index {
            return true;
        }
        let active_id = self.active_tab().map(|t| t.id);
        let tab = self.tabs.remove(from_index);
        self.tabs.insert(to_index, tab);
        if let Some(id) = active_id {
            if let Some(new_pos) = self.tabs.iter().position(|t| t.id == id) {
                self.active_index = new_pos;
            }
        }
        true
    }

    pub fn update_active_url(&mut self, url: Url) {
        if let Some(tab) = self.active_tab_mut() {
            tab.title = Tab::derive_title_from_url(&url);
            tab.url = url;
        }
    }

    pub fn update_active_title(&mut self, title: String) {
        if let Some(tab) = self.active_tab_mut() {
            tab.title = title;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_lifecycle_and_switching() {
        let mut manager = TabManager::new(Url::parse("https://first.com/").unwrap());
        assert_eq!(manager.len(), 1);
        assert_eq!(manager.active_tab().unwrap().title, "first.com");

        let id2 = manager.open_tab(Url::parse("https://second.org/").unwrap());
        let id3 = manager.open_tab(Url::parse("https://third.net/").unwrap());
        assert_eq!(manager.len(), 3);
        assert_eq!(manager.active_tab().unwrap().id, id3);

        assert!(manager.select_tab(id2));
        assert_eq!(manager.active_index(), 1);
        assert!(manager.select_index(0));
        assert_eq!(manager.active_index(), 0);
    }

    #[test]
    fn test_tab_closing_variants() {
        let mut manager = TabManager::new(Url::parse("https://a.com/").unwrap());
        let id_b = manager.open_tab(Url::parse("https://b.com/").unwrap());
        let id_c = manager.open_tab(Url::parse("https://c.com/").unwrap());

        // Close middle tab while active
        manager.select_tab(id_b);
        assert_eq!(
            manager.close_tab(id_b),
            CloseTabResult::Closed {
                new_active: Some(id_c)
            }
        );
        assert_eq!(manager.len(), 2);
        assert_eq!(manager.active_tab().unwrap().id, id_c);

        // Close last tab while active -> shifts active to preceding tab
        assert_eq!(
            manager.close_tab(id_c),
            CloseTabResult::Closed {
                new_active: Some(TabId(1))
            }
        );
        assert_eq!(manager.len(), 1);

        // Close remaining tab -> NoTabsRemaining
        assert_eq!(manager.close_tab(TabId(1)), CloseTabResult::NoTabsRemaining);
        assert!(manager.is_empty());
    }

    #[test]
    fn test_tab_reordering_and_truncation() {
        let mut manager = TabManager::new(Url::parse("https://a.com/").unwrap());
        let _id_b = manager.open_tab(Url::parse("https://b.com/").unwrap());
        let id_c = manager.open_tab(Url::parse("https://c.com/").unwrap());

        assert!(manager.move_tab(0, 2));
        assert_eq!(manager.tabs()[0].url.as_str(), "https://b.com/");
        assert_eq!(manager.tabs()[1].url.as_str(), "https://c.com/");
        assert_eq!(manager.active_tab().unwrap().id, id_c);

        let long_url =
            Url::parse("https://very-long-domain-name-exceeding-limits.example.com/").unwrap();
        let tab = Tab::new(TabId(99), long_url);
        assert_eq!(tab.display_title(15), "very-long-do...");
    }
}
