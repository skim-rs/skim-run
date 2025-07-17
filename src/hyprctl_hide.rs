use std::{borrow::Cow, process::Command, sync::Arc};

use anyhow::Result;
use serde::Deserialize;
use skim::{ItemPreview, PreviewContext, SkimItem, SkimOutput};

use crate::SkimRun;

/// Represents a Hyprland client/window.
#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct Client {
    address: String,
    class: String,
    title: String,
    hidden: bool,
    mapped: bool,
    focusHistoryID: u16,
    workspace: Workspace,
}

#[derive(Deserialize, Debug)]
struct Workspace {
    name: String,
}

/// `SkimItem` wrapper for a Hyprland client.
struct ClassWindow {
    client: Client,
}

impl SkimItem for ClassWindow {
    fn text(&self) -> Cow<str> {
        Cow::Owned(format!(
            "{} [{}]{}{}",
            self.client.title,
            self.client.class,
            if self.client.hidden { " (hidden)" } else { "" },
            if self.client.mapped {
                ""
            } else {
                " (unmapped)"
            }
        ))
    }
    fn output(&self) -> Cow<str> {
        Cow::Borrowed(&self.client.address)
    }
    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        ItemPreview::Text(format!(
            "Title: {}\nClass: {}\nWorkspace: {}\nAddress: {}\nHidden: {}\nMapped: {}\nFocusHistoryID: {}",
            self.client.title,
            self.client.class,
            self.client.workspace.name,
            self.client.address,
            self.client.hidden,
            self.client.mapped,
            self.client.focusHistoryID
        ))
    }
}

/// Mode for listing all windows in the special hidden workspace (id 99).
/// Lets the user swap the selected hidden window with the current one (enter),
/// or simply unhide it (shift-enter).
#[derive(Default, Clone)]
pub struct HyprctlHide;

impl SkimRun for HyprctlHide {
    fn get(&self) -> Vec<Arc<dyn SkimItem>> {
        let res = Command::new("hyprctl")
            .arg("clients")
            .arg("-j")
            .output()
            .expect("Failed to get hyprctl clients")
            .stdout;
        let clients: Vec<Client> =
            serde_json::from_slice(&res).expect("Failed to parse clients from JSON");

        // List all windows in the special:hidden workspace
        clients
            .into_iter()
            .filter(|c| c.workspace.name == "special:hidden")
            .map(|c| Arc::new(ClassWindow { client: c }) as Arc<dyn SkimItem>)
            .collect()
    }

    fn set_options(&self, opts: &mut skim::SkimOptions) {
        opts.preview = Some(String::new());
        opts.header = Some("List of hidden windows (special:hidden). Enter: swap with current. Shift-Enter: unhide.".to_string());
        opts.preview_window = String::from("up:40%");
        opts.bind.extend(vec![
            // Enter: swap current window with selected hidden window
            // 1. Move current window to special:hidden
            // 2. Move selected window to current workspace
            "enter:execute(hyprctl activewindow -j | jq -r .address | xargs -I{} hyprctl dispatch movetoworkspacesilent special:hidden,address:{} ; hyprctl activeworkspace -j | jq -r .id | xargs -I{ws} hyprctl dispatch movetoworkspacesilent {ws},address:{} ; hyprctl dispatch focuswindow address:{})+accept".to_string(),
            // Alt-Enter: unhide selected window (move to current workspace and focus)
            "alt-enter:execute(hyprctl activeworkspace -j | jq -r .id | xargs -I{ws} hyprctl dispatch movetoworkspacesilent {ws},address:{} ; hyprctl dispatch focuswindow address:{})+accept".to_string(),
        ]);
    }

    fn run(&self, _output: &SkimOutput) -> Result<()> {
        // No-op: actions are handled by execute binds.
        Ok(())
    }
}
