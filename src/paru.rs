use std::{borrow::Cow, io::BufRead, process::Command, sync::Arc};

use skim::{ItemPreview, SkimItem};

use crate::SkimRun;

struct Package {
    repo: String,
    name: String,
    version: String,
}

impl SkimItem for Package {
    fn text(&self) -> std::borrow::Cow<str> {
        Cow::Owned(format!("[{}] {} ({})", self.repo, self.name, self.version))
    }
    fn preview(&self, _context: skim::PreviewContext) -> skim::ItemPreview {
        ItemPreview::Command(format!("paru -Si {}/{}", self.repo, self.name))
    }
}

pub struct Paru;

impl SkimRun for Paru {
    fn get(&self) -> Vec<Arc<dyn SkimItem>> {
        Command::new("paru")
            .args(["-S", "--list"])
            .output()
            .expect("Failed to list packages")
            .stdout
            .lines()
            .map(|l| {
                let a = l.unwrap_or_default();
                let mut parts = a.split(' ');
                let repo = parts.next().unwrap_or_default().to_string();
                let name = parts.next().unwrap_or_default().to_string();
                let version = parts.next().unwrap_or_default().to_string();
                Arc::new(Package {
                    repo,
                    name,
                    version,
                }) as Arc<dyn SkimItem>
            })
            .collect()
    }
    fn set_options(&self, opts: &mut skim::SkimOptions) {
        opts.preview = Some(String::new());
        opts.delimiter = String::from(r"[ \t\[\]()]+");
        opts.bind.extend_from_slice(&[
            "ctrl-i:execute(paru -S --needed '{2}/{3}')".to_string(),
            "ctrl-r:execute(paru -Rs '{2}/{3}')".to_string(),
        ]);
    }
}
