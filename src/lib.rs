use anyhow::Result;
use std::sync::Arc;

use skim::{SkimItem, SkimOutput, prelude::SkimOptionsBuilder};

#[cfg(feature = "apps")]
pub mod apps;
#[cfg(feature = "calc")]
pub mod calc;
pub mod cli;
#[cfg(feature = "hyprland")]
pub mod hyprctl_clients;
#[cfg(feature = "paru")]
pub mod paru;
#[cfg(feature = "systemd")]
pub mod systemd_services;
pub use cli::*;

pub trait SkimRun {
    ///! Init the runner
    ///! Will return false if we should stop here, or true if the skim instance should be started
    fn init(&self, args: &Mode) -> bool {
        let _ = args;
        return true;
    }

    ///! Get Items
    fn get(&self) -> Vec<Arc<dyn SkimItem>> {
        Vec::new()
    }

    ///! Set SkimOptions
    fn set_options<'a>(&self, opts: &'a mut skim::SkimOptions) {
        let _ = opts;
    }

    ///! Run on the result from skim
    fn run(&self, output: &SkimOutput) -> Result<()> {
        let _ = output;
        Ok(())
    }
}
