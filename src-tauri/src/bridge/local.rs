//! Shell-only native caller scope: no service/bootstrap/transport authority.
use super::*;
#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub enum Request {
    Status {},
    Retry {},
    Acknowledge { revision: String },
    Enter {},
}
#[derive(Clone)]
pub struct Permit {
    bridge: Arc<Bridge>,
    binding: Binding,
}
impl Permit {
    pub(crate) fn with_current<T>(&self, work: impl FnOnce() -> T) -> Result<T, Denied> {
        let state = self.bridge.state.lock().map_err(|_| Denied)?;
        if state.owner != Some(self.binding) || self.binding.domain != Domain::Shell {
            return Err(Denied);
        }
        Ok(work())
    }
}
#[cfg(any(test, feature = "native-bridge"))]
impl Bridge {
    fn admit_local(self: &Arc<Self>, binding: Binding, sender: &Sender) -> Result<Permit, Denied> {
        let state = self.state.lock().map_err(|_| Denied)?;
        if binding.domain != Domain::Shell {
            return Err(Denied);
        }
        validate_owned_sender(&state, binding, sender, "shell")?;
        Ok(Permit {
            bridge: self.clone(),
            binding,
        })
    }
}
#[cfg(all(target_os = "macos", feature = "native-bridge"))]
pub mod macos;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn local_metadata_admission_needs_owned_shell_main_frame_but_not_auth_notice() {
        let (bridge, _, mut sender, _) = crate::bridge::tests::fixture();
        bridge.set_runtime(7, false, false).unwrap();
        let binding = bridge.bind_owned(41, Domain::Shell).unwrap();
        sender.host = "shell".into();
        sender.frame_url = Some("dsh-app://shell/".into());
        sender.view_url = sender.frame_url.clone();
        let scope = bridge.admit_local(binding, &sender).unwrap();
        assert!(scope.with_current(|| ()).is_ok());
        sender.main_frame = false;
        assert!(bridge.admit_local(binding, &sender).is_err());
        sender.main_frame = true;
        sender.host = "app".into();
        assert!(bridge.admit_local(binding, &sender).is_err());
        bridge.detach(binding);
        assert!(scope.with_current(|| panic!("late shell reply")).is_err());
    }
    #[test]
    fn shell_actions_are_closed_and_cannot_forward_or_select_native_authority() {
        for input in [
            r#"{"action":"fetch","path":"/api/tasks"}"#,
            r#"{"action":"status","window":"primary"}"#,
            r#"{"action":"acknowledge","revision":"x","cookie":"secret"}"#,
        ] {
            assert!(serde_json::from_str::<Request>(input).is_err());
        }
    }
}
