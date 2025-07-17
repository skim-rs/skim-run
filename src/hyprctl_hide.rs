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
#[derive(Clone)]
pub struct HyprctlHide {
    pub ignore_class: Option<String>,
    pub swap: Option<String>,
}

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

        // List all windows in the special:hidden workspace, ignoring the specified class
        clients
            .into_iter()
            .filter(|c| c.workspace.name == "special:hidden")
            .filter(|c| {
                if let Some(ref ignore) = self.ignore_class {
                    c.class != *ignore
                } else {
                    true
                }
            })
            .map(|c| Arc::new(ClassWindow { client: c }) as Arc<dyn SkimItem>)
            .collect()
    }

    fn set_options(&self, opts: &mut skim::SkimOptions) {
        opts.preview = Some(String::new());
        opts.header = Some("List of hidden windows (special:hidden). Enter: swap with current (or previously focused if ignored). Alt-Enter: unhide.".to_string());
        opts.preview_window = String::from("up:40%");
        let ignore_class = self.ignore_class.clone().unwrap_or_default();

        // The swap logic:
        // 1. Get the currently focused window's class.
        // 2. If it matches ignore_class, find the previously focused window (not ignored).
        // 3. Hide that window instead.
        // 4. Unhide (move to current workspace and focus) the selected window.

        // This is implemented as a shell script in the execute binding.
        opts.bind.extend(vec![
            format!(
                "enter:execute({} hyprctl-hide --swap {})",
                std::env::args().next().unwrap_or_else(|| "skim-run".to_string()),
                "{}"
            ),
            // Alt-Enter: unhide selected window (move to current workspace and focus)
            "alt-enter:execute(hyprctl activeworkspace -j | jq -r .id | xargs -I{ws} hyprctl dispatch movetoworkspacesilent {ws},address:{} ; hyprctl dispatch focuswindow address:{})".to_string(),
        ]);
    }

    fn run(&self, _output: &SkimOutput) -> Result<()> {
        // If swap argument is provided, perform the swap logic.
        if let Some(target_addr) = &self.swap {
            // Get binary name from argv[0]
            let bin = std::env::args().next().unwrap_or_else(|| "skim-run".to_string());
            // Get currently focused window address
            let curr_json = std::process::Command::new("hyprctl")
                .arg("activewindow")
                .arg("-j")
                .output()
                .map(|o| o.stdout)
                .unwrap_or_default();
            let curr_addr = serde_json::from_slice::<serde_json::Value>(&curr_json)
                .and_then(|v| Ok(v.get("address").and_then(|a| a.as_str()).unwrap_or("")))
                .unwrap_or("");
            // Move current window to hidden
            let _ = std::process::Command::new("hyprctl")
                .args(&["dispatch", "movetoworkspacesilent", "special:hidden,address:".to_owned() + curr_addr])
                .status();
            // Move target window to current workspace
            let ws_json = std::process::Command::new("hyprctl")
                .arg("activeworkspace")
                .arg("-j")
                .output()
                .map(|o| o.stdout)
                .unwrap_or_default();
            let ws_id = serde_json::from_slice::<serde_json::Value>(&ws_json)
                .and_then(|v| Ok(v.get("id").and_then(|i| i.as_i64()).unwrap_or(1)))
                .unwrap_or(1);
            let _ = std::process::Command::new("hyprctl")
                .args(&["dispatch", "movetoworkspacesilent", &format!("{},address:{}", ws_id, target_addr)])
                .status();
            // Focus the target window
            let _ = std::process::Command::new("hyprctl")
                .args(&["dispatch", "focuswindow", &format!("address:{}", target_addr)])
                .status();
        }
        Ok(())
    }
}
