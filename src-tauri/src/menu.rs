//! Menu accelerators and clicks route through the same native command identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Close,
    Quit,
    Open,
    About,
    Safety,
}
pub fn command(id: &str) -> Option<Command> {
    match id {
        "desktop.close" => Some(Command::Close),
        "desktop.quit" => Some(Command::Quit),
        "desktop.open" => Some(Command::Open),
        "desktop.about" => Some(Command::About),
        "desktop.safety" => Some(Command::Safety),
        _ => None,
    }
}
#[cfg(feature = "desktop")]
pub mod native {
    use super::*;
    use crate::{bundle::Manifest, quit::native::Quit, window::native::Primary};
    use std::sync::Arc;
    use tauri::{
        AppHandle,
        menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    };
    use tauri_plugin_dialog::DialogExt;
    pub fn build(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
        let close = MenuItem::with_id(app, "desktop.close", "Close Window", true, Some("Cmd+W"))?;
        let quit = MenuItem::with_id(
            app,
            "desktop.quit",
            "Quit DeepSeek Harness Desktop…",
            true,
            Some("Cmd+Q"),
        )?;
        let open = MenuItem::with_id(
            app,
            "desktop.open",
            "Open Workspace Window",
            true,
            None::<&str>,
        )?;
        let about = MenuItem::with_id(
            app,
            "desktop.about",
            "About DeepSeek Harness Desktop",
            true,
            None::<&str>,
        )?;
        let safety = MenuItem::with_id(app, "desktop.safety", "Safety Notice", true, None::<&str>)?;
        let separator = PredefinedMenuItem::separator(app)?;
        let application = Submenu::with_items(
            app,
            "DeepSeek Harness Desktop",
            true,
            &[&about, &safety, &separator, &quit],
        )?;
        let file = Submenu::with_items(app, "File", true, &[&open, &close])?;
        let edit = Submenu::with_items(
            app,
            "Edit",
            true,
            &[
                &PredefinedMenuItem::undo(app, None)?,
                &PredefinedMenuItem::redo(app, None)?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::cut(app, None)?,
                &PredefinedMenuItem::copy(app, None)?,
                &PredefinedMenuItem::paste(app, None)?,
                &PredefinedMenuItem::select_all(app, None)?,
            ],
        )?;
        Menu::with_items(app, &[&application, &file, &edit])
    }
    pub fn dispatch(
        app: &AppHandle,
        id: &str,
        primary: &Arc<Primary>,
        quit: &Arc<Quit>,
        manifest: &Manifest,
    ) {
        match command(id) {
            Some(Command::Close) => {
                let _ = primary.close(app);
            }
            Some(Command::Quit) => quit.request(),
            Some(Command::Open) => {
                let _ = primary.open(app);
            }
            Some(Command::About) => {
                app.dialog()
                    .message(format!(
                        "Desktop {}\ndsh {}\nLocal ad-hoc experimental build.",
                        manifest.desktop_version, manifest.dsh_version
                    ))
                    .title("DeepSeek Harness Desktop")
                    .show(|_| {});
            }
            Some(Command::Safety) => {
                app.dialog().message("Experimental software. Review workspace permissions and every requested action. Harness can read and change files and run commands. Model credentials use upstream plaintext storage with owner-only permissions. Local bearer authentication does not isolate other processes running as your user. Closing the window keeps the service running; quitting stops work across all sessions and workspaces.").title("Safety Notice").show(|_|{});
            }
            None => {}
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_closed_native_commands_route() {
        for (id, expected) in [
            ("desktop.close", Command::Close),
            ("desktop.quit", Command::Quit),
            ("desktop.open", Command::Open),
            ("desktop.about", Command::About),
            ("desktop.safety", Command::Safety),
        ] {
            assert_eq!(command(id), Some(expected));
        }
        assert_eq!(command("renderer.run"), None);
    }
}
