use std::{borrow::Cow, cmp::min, ops::Deref, process::Command, thread::sleep, time::Duration};

use anyhow::Context as _;
use applications::{AppInfo, AppInfoContext, common::SearchPath};
use fork::daemon;
use skim::prelude::*;

use crate::SkimRun;

#[derive(Default)]
pub struct App {
    app: applications::App,
}
impl Deref for App {
    type Target = applications::App;

    fn deref(&self) -> &Self::Target {
        &self.app
    }
}
impl From<applications::App> for App {
    fn from(value: applications::App) -> Self {
        Self { app: value }
    }
}

impl SkimItem for App {
    fn text(&self) -> Cow<str> {
        if self.app.name.is_empty() {
            Cow::Borrowed(
                self.app
                    .app_desktop_path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .strip_suffix(".desktop")
                    .unwrap(),
            )
        } else {
            Cow::Borrowed(&self.app.name)
        }
    }
    fn preview(&self, context: PreviewContext) -> ItemPreview {
        let width = u32::try_from(context.width).unwrap_or(16);
        let height = u32::try_from(context.height).unwrap_or(16);
        let size = min(16, min(height, width));
        if self.icon_path.is_some() {
            let conf = viuer::Config {
                width: Some(size),
                x: 0,
                y: 0,
                transparent: true,
                restore_cursor: false,
                use_kitty: false,
                truecolor: true,
                ..Default::default()
            };
            let img = image::DynamicImage::ImageRgb8(image::RgbImage::new(size, size));
            let _ = viuer::print(&img, &conf);
            let _ = viuer::print_from_file(self.icon_path.clone().unwrap(), &conf);
        }

        ItemPreview::Text(String::new())
    }
    fn output(&self) -> Cow<str> {
        if self.app_path_exe.is_some() {
            self.app_path_exe
                .as_ref()
                .unwrap()
                .as_path()
                .to_string_lossy()
        } else {
            Cow::default()
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct Apps;

impl SkimRun for Apps {
    fn get(&self) -> Vec<std::sync::Arc<(dyn skim::SkimItem + 'static)>> {
        let mut ctx = AppInfoContext::new(vec![SearchPath::new(std::path::PathBuf::from("/"), 1)]);
        ctx.refresh_apps().unwrap(); // must refresh apps before getting them

        ctx.get_all_apps()
            .into_iter()
            .map(|a| Arc::new(App::from(a)) as Arc<dyn SkimItem>)
            .collect()
    }
    fn run(&self, output: &SkimOutput) -> anyhow::Result<()> {
        let item = output
            .selected_items
            .first()
            .context("Could not find selected item")?;
        if let Ok(fork::Fork::Child) = daemon(false, false) {
            Command::new(item.output().into_owned())
                .spawn()
                .context("Failed to spawn app")?;
            sleep(Duration::from_secs(10));
        }
        Ok(())
    }
    fn set_options(&self, opts: &mut SkimOptions) {
        opts.preview = Some(String::new());
        opts.preview_window = String::from("left:16");
    }
}
