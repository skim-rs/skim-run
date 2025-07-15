use anyhow::Result;
use std::sync::Arc;

use skim::{SkimItem, SkimOutput, prelude::SkimOptionsBuilder};

pub mod apps;
pub mod calc;
pub mod cli;
pub mod hyprctl_clients;
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
    fn set_options<'a>(&self, opts: &'a mut SkimOptionsBuilder) -> &'a mut SkimOptionsBuilder {
        return opts;
    }

    ///! Run on the result from skim
    fn run(&self, output: &SkimOutput) -> Result<()> {
        let _ = output;
        Ok(())
    }
}
