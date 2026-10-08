//! Shell window identity/policy, never task or runtime state.
use url::Url;
#[derive(Debug, PartialEq, Eq)]
pub enum WindowAction {
    None,
    Create,
    Show(u64),
    Hide(u64),
}
#[derive(Default)]
pub struct WindowState {
    identity: Option<u64>,
    last: u64,
    creating: bool,
    failed: bool,
}
impl WindowState {
    pub fn open(&mut self) -> WindowAction {
        if let Some(id) = self.identity
            && !self.failed
        {
            return WindowAction::Show(id);
        }
        if self.creating {
            return WindowAction::None;
        }
        self.creating = true;
        WindowAction::Create
    }
    pub fn opened(&mut self, id: u64) {
        if id <= self.last {
            return;
        }
        self.last = id;
        self.identity = Some(id);
        self.failed = false;
        self.creating = false;
    }
    pub fn creation_failed(&mut self) {
        self.creating = false;
    }
    pub fn close(&self) -> WindowAction {
        self.identity.map_or(WindowAction::None, WindowAction::Hide)
    }
    pub fn failed(&mut self, id: u64) {
        if self.identity == Some(id) {
            self.failed = true;
            self.creating = false;
        }
    }
    pub fn destroyed(&mut self, id: u64) {
        if self.identity == Some(id) {
            self.identity = None;
            self.failed = false;
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum Navigation {
    Owned,
    External,
    Deny,
}
pub fn navigation(url: &Url, user_link: bool) -> Navigation {
    if !url.username().is_empty() || url.password().is_some() {
        return Navigation::Deny;
    }
    if url.scheme() == "dsh-app"
        && matches!(url.host_str(), Some("app" | "shell"))
        && url.port().is_none()
    {
        Navigation::Owned
    } else if user_link && matches!(url.scheme(), "http" | "https") && url.host_str().is_some() {
        Navigation::External
    } else {
        Navigation::Deny
    }
}
#[cfg(feature = "desktop")]
pub mod native;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn creation_is_joined_and_late_open_or_loss_cannot_replace_identity() {
        let mut s = WindowState::default();
        assert_eq!(s.open(), WindowAction::Create);
        assert_eq!(s.open(), WindowAction::None);
        s.opened(1);
        assert_eq!(s.close(), WindowAction::Hide(1));
        assert_eq!(s.open(), WindowAction::Show(1));
        s.failed(1);
        assert_eq!(s.open(), WindowAction::Create);
        s.opened(2);
        s.opened(1);
        s.destroyed(1);
        assert_eq!(s.open(), WindowAction::Show(2));
    }
    #[test]
    fn only_native_user_links_open_external_http_without_authority() {
        for raw in ["https://example.test/path?q=x", "http://example.test/"] {
            let u = Url::parse(raw).unwrap();
            assert_eq!(navigation(&u, false), Navigation::Deny);
            assert_eq!(navigation(&u, true), Navigation::External);
        }
        for raw in [
            "file:///tmp/key",
            "data:text/html,x",
            "javascript:alert(1)",
            "dsh-app://evil/",
            "https://secret@example.test/",
        ] {
            assert_eq!(
                navigation(&Url::parse(raw).unwrap(), true),
                Navigation::Deny
            );
        }
        assert_eq!(
            navigation(
                &Url::parse("dsh-app://app/#desktop-window=1").unwrap(),
                false
            ),
            Navigation::Owned
        );
    }
}
