use std::{borrow::Cow, process::Command, sync::Arc};

use anyhow::Context as _;
use serde::Deserialize;
use skim::SkimItem;

use crate::SkimRun;

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct Client {
    title: String,
    focusHistoryID: u16,
    address: String,
}

pub struct HyprctlClients;

impl SkimRun for HyprctlClients {
    fn get(&self) -> Vec<std::sync::Arc<dyn SkimItem>> {
        let res = Command::new("hyprctl")
            .arg("clients")
            .arg("-j")
            .output()
            .expect("Failed to get hyprcl clients")
            .stdout;
        let mut clients: Vec<Client> =
            serde_json::from_slice(&res).expect("Failed to parse clients from JSON");
        clients.sort_unstable_by(|a, b| a.focusHistoryID.cmp(&b.focusHistoryID));
        clients
            .into_iter()
            .filter_map(|c| {
                if c.focusHistoryID == 0 {
                    return None;
                }
                Some(Arc::new(c) as Arc<dyn SkimItem>)
            })
            .collect()
    }
    fn run(&self, output: &skim::SkimOutput) -> anyhow::Result<()> {
        let result = output
            .selected_items
            .first()
            .expect("Failed to get selected item");
        let _ = Command::new("hyprctl")
            .arg("dispatch")
            .arg("focuswindow")
            .arg(format!("address:{}", result.output()))
            .spawn()
            .context("Failed to run hyprctl dispatch focuswindow")?
            .wait();
        Ok(())
    }
}

impl SkimItem for Client {
    fn text(&self) -> std::borrow::Cow<str> {
        Cow::Borrowed(&self.title)
    }
    fn output(&self) -> std::borrow::Cow<str> {
        Cow::Borrowed(&self.address)
    }
}
