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
                "enter:execute(
                    bash -c '
                        IGNORE_CLASS=\"{ignore_class}\"
                        CURR=$(hyprctl activewindow -j)
                        CURR_ADDR=$(echo \"$CURR\" | jq -r .address)
                        CURR_CLASS=$(echo \"$CURR\" | jq -r .class)
                        if [ \"$CURR_CLASS\" = \"$IGNORE_CLASS\" ]; then
                            # Find previously focused window not ignored
                            PREV=$(hyprctl clients -j | jq -c \".[] | select(.class != \\\"$IGNORE_CLASS\\\")\" | jq -s \"sort_by(.focusHistoryID) | reverse | .[1]\")
                            if [ \"$PREV\" != \"null\" ]; then
                                PREV_ADDR=$(echo \"$PREV\" | jq -r .address)
                                hyprctl dispatch movetoworkspacesilent special:hidden,address:$PREV_ADDR
                            fi
                        else
                            hyprctl dispatch movetoworkspacesilent special:hidden,address:$CURR_ADDR
                        fi
                        WS=$(hyprctl activeworkspace -j | jq -r .id)
                        hyprctl dispatch movetoworkspacesilent $WS,address:{}
                        hyprctl dispatch focuswindow address:{}
                    '
                )",
            ),
            // Alt-Enter: unhide selected window (move to current workspace and focus)
            "alt-enter:execute(hyprctl activeworkspace -j | jq -r .id | xargs -I{ws} hyprctl dispatch movetoworkspacesilent {ws},address:{} ; hyprctl dispatch focuswindow address:{})".to_string(),
        ]);
    }

    fn run(&self, _output: &SkimOutput) -> Result<()> {
        // No-op: actions are handled by execute binds.
        Ok(())
    }
}
