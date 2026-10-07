use serde::Serialize;
use std::fmt;

// Only these fixed categories cross the native/local-view boundary. Never wrap raw logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Failure {
    InvalidBundle,
    Launch,
    Announcement,
    Timeout,
    Exited,
    Cleanup,
    Connection,
    Unavailable,
}

impl Failure {
    pub fn description(self) -> &'static str {
        match self {
            Self::InvalidBundle => "The installed runtime is missing, damaged or incompatible.",
            Self::Launch => {
                "The local runtime could not start or access its private data directory."
            }
            Self::Announcement => "The local runtime did not provide a valid private connection.",
            Self::Timeout => "The local runtime did not finish connecting within 15 seconds.",
            Self::Exited => {
                "The local runtime exited unexpectedly. Unfinished work may be interrupted."
            }
            Self::Cleanup => {
                "Owned runtime cleanup could not be confirmed. Restart and quit are blocked."
            }
            Self::Connection => {
                "The private local connection could not be authenticated or prepared."
            }
            Self::Unavailable => "This runtime operation is no longer available.",
        }
    }

    pub fn next_action(self) -> &'static str {
        match self {
            Self::InvalidBundle => "Reinstall a matching build before trying again.",
            Self::Launch => "Check application-data permissions, then choose Retry.",
            Self::Announcement | Self::Timeout | Self::Connection => {
                "Choose Retry after owned cleanup completes."
            }
            Self::Exited => {
                "Choose Retry to reconnect. Previous requests and approvals will not be replayed."
            }
            Self::Cleanup => {
                "Keep the application open and retry cleanup. Do not launch a competing runtime."
            }
            Self::Unavailable => "Use the current application window and runtime state.",
        }
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.description())
    }
}
impl std::error::Error for Failure {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_are_fixed_categories_with_actions() {
        for failure in [
            Failure::InvalidBundle,
            Failure::Launch,
            Failure::Announcement,
            Failure::Timeout,
            Failure::Exited,
            Failure::Cleanup,
            Failure::Connection,
            Failure::Unavailable,
        ] {
            assert!(!failure.description().is_empty());
            assert!(!failure.next_action().is_empty());
            let json = serde_json::to_string(&failure).unwrap();
            assert!(!json.contains("token"));
            assert!(!json.contains("http"));
            assert_eq!(failure.to_string(), failure.description());
        }
    }
}
