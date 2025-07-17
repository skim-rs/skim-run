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
        eprintln!("[hyprctl-hide][get] ignore_class: {:?}", self.ignore_class);
        let res = Command::new("hyprctl")
            .arg("clients")
            .arg("-j")
            .output()
            .expect("Failed to get hyprctl clients")
            .stdout;
        let clients: Vec<Client> =
            serde_json::from_slice(&res).expect("Failed to parse clients from JSON");

        // List all windows in the special:hidden workspace, ignoring the specified class,
        // and sort by recency (focusHistoryID ascending)
        let mut filtered: Vec<Client> = clients
            .into_iter()
            .filter(|c| c.workspace.name == "special:hidden")
            .filter(|c| {
                if let Some(ref ignore) = self.ignore_class {
                    c.class != *ignore
                } else {
                    true
                }
            })
            .collect();
        filtered.sort_by_key(|c| c.focusHistoryID);
        filtered
            .into_iter()
            .map(|c| Arc::new(ClassWindow { client: c }) as Arc<dyn SkimItem>)
            .collect()
    }

    fn set_options(&self, opts: &mut skim::SkimOptions) {
        eprintln!(
            "[hyprctl-hide][set_options] ignore_class: {:?}",
            self.ignore_class
        );
        opts.preview = Some(String::new());
        opts.header = Some("List of hidden windows (special:hidden). Enter: swap with current (or previously focused if ignored). Alt-Enter: unhide.".to_string());
        opts.preview_window = String::from("up:40%");

        // The swap logic:
        // 1. Get the currently focused window's class.
        // 2. If it matches ignore_class, find the previously focused window (not ignored).
        // 3. Hide that window instead.
        // 4. Unhide (move to current workspace and focus) the selected window.

        // This is implemented as a shell script in the execute binding.
        let ignore_class_arg = if let Some(ref ignore) = self.ignore_class {
            format!(" --ignore-class {}", ignore)
        } else {
            String::new()
        };
        opts.bind.extend(vec![
            format!(
                "enter:execute({} hyprctl-hide{} --swap {})+accept",
                std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_else(|_| "skim-run".to_string()),
                ignore_class_arg,
                "{}"
            ),
            // Alt-Enter: unhide selected window (move to current workspace and focus)
            "alt-enter:execute(hyprctl activeworkspace -j | jq -r .id | xargs -I{ws} hyprctl dispatch movetoworkspacesilent {ws},address:{} ; hyprctl dispatch focuswindow address:{})+accept".to_string(),
        ]);
    }

    fn run(&self, _output: &SkimOutput) -> Result<()> {
        // No-op: swap logic is handled in init()
        Ok(())
    }
    fn init(&self, mode: &crate::Mode) -> bool {
        // If swap argument is provided, perform the swap logic and exit.
        if let crate::Mode::HyprctlHide {
            swap: Some(target_addr),
            ..
        } = mode
        {
            // Get currently focused window address and class
            eprintln!("[hyprctl-hide][swap] Fetching currently focused window address...");
            let curr_json = std::process::Command::new("hyprctl")
                .arg("activewindow")
                .arg("-j")
                .output()
                .map(|o| o.stdout)
                .unwrap_or_default();
            let curr_val =
                serde_json::from_slice::<serde_json::Value>(&curr_json).unwrap_or_default();
            let curr_addr = curr_val
                .get("address")
                .and_then(|a| a.as_str())
                .unwrap_or("")
                .to_string();
            let curr_class = curr_val
                .get("class")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string();
            eprintln!(
                "[hyprctl-hide][swap] Current window address: '{}'",
                curr_addr
            );
            eprintln!(
                "[hyprctl-hide][swap] Current window class: '{}'",
                curr_class
            );
            eprintln!(
                "[hyprctl-hide][swap] Target window address: '{}'",
                target_addr
            );

            // If ignore_class is set and current window matches, hide previously focused window instead
            let mut hide_addr = curr_addr.clone();
            if let Some(ref ignore) = self.ignore_class {
                if curr_class == *ignore {
                    eprintln!(
                        "[hyprctl-hide][swap] Current window matches ignore_class ('{}'), searching for previously focused window to hide...",
                        ignore
                    );
                    // Get all clients and sort by focusHistoryID descending, skip ignored class and current
                    let clients_json = std::process::Command::new("hyprctl")
                        .arg("clients")
                        .arg("-j")
                        .output()
                        .map(|o| o.stdout)
                        .unwrap_or_default();
                    let clients: Vec<Client> =
                        serde_json::from_slice(&clients_json).unwrap_or_default();
                    if let Some(prev) = clients
                        .iter()
                        .filter(|c| {
                            c.class != *ignore && c.address != curr_addr && c.focusHistoryID > 0
                        })
                        .min_by_key(|c| c.focusHistoryID)
                    {
                        hide_addr = prev.address.clone();
                        eprintln!(
                            "[hyprctl-hide][swap] Hiding previously focused window: '{}', class: '{}', focusHistoryID: {}",
                            hide_addr, prev.class, prev.focusHistoryID
                        );
                    } else {
                        eprintln!(
                            "[hyprctl-hide][swap] No suitable previously focused window found, will not hide any window."
                        );
                        hide_addr.clear();
                    }
                }
            }

            // Move window to hidden if we have an address
            if !hide_addr.is_empty() {
                let hide_cmd = format!(
                    "hyprctl dispatch movetoworkspacesilent special:hidden,address:{}",
                    hide_addr
                );
                eprintln!("[hyprctl-hide][swap] Executing: {}", hide_cmd);
                let _ = std::process::Command::new("hyprctl")
                    .args(&[
                        "dispatch",
                        "movetoworkspacesilent",
                        &format!("special:hidden,address:{}", hide_addr),
                    ])
                    .status();
            }

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
            eprintln!("[hyprctl-hide][swap] Current workspace id: {}", ws_id);

            let bring_cmd = format!(
                "hyprctl dispatch movetoworkspacesilent {},address:{}",
                ws_id, target_addr
            );
            eprintln!("[hyprctl-hide][swap] Executing: {}", bring_cmd);
            let _ = std::process::Command::new("hyprctl")
                .args(&[
                    "dispatch",
                    "movetoworkspacesilent",
                    &format!("{},address:{}", ws_id, target_addr),
                ])
                .status();

            let focus_cmd = format!("hyprctl dispatch focuswindow address:{}", target_addr);
            eprintln!("[hyprctl-hide][swap] Executing: {}", focus_cmd);
            let _ = std::process::Command::new("hyprctl")
                .args(&[
                    "dispatch",
                    "focuswindow",
                    &format!("address:{}", target_addr),
                ])
                .status();
            // Exit immediately, do not start TUI
            eprintln!("[hyprctl-hide][swap] Swap complete, exiting.");
            return false;
        }
        true
    }
}
