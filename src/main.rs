use anyhow::{Context as _, Result};
use clap::Parser;
use skim::prelude::*;
use skim_run::*;

fn run_with(mode: &Box<dyn SkimRun>, args: &Cli) -> Result<Option<String>> {
    let mut options_builder = SkimOptionsBuilder::default();

    let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();

    let mut options_builder = mode
        .set_options(&mut options_builder)
        .query(args.query.clone());
    if !args.modes.is_sorted() {
        let current_mode_idx = args
            .modes
            .iter()
            .position(|m| m.to_lowercase() == format!("{:?}", args.mode).to_lowercase())
            .context("Current mode not found in enabled modes")?;

        let next_mode = args.modes[(current_mode_idx + 1) % args.modes.len()].clone();
        options_builder = options_builder.bind(vec![format!("tab:accept({})", next_mode)]);
    }
    let options = options_builder
        .build()
        .context("Failed to build skim options")?;
    for item in mode.get() {
        let _ = tx_item.send(item);
    }
    drop(tx_item);

    let Some(output) = Skim::run_with(&options, Some(rx_item)) else {
        return Ok(None);
    };
    if output.is_abort {
        return Ok(None);
    }

    let _ = match output.final_event {
        Event::EvActAccept(Some(ref next_args)) => {
            let _ = mode.run(&output);
            let mut next_args = next_args.replace("{q}", &output.query);
            next_args = next_args.replace("{cq}", &output.cmd);
            next_args = next_args.replace(
                "{}",
                &output
                    .selected_items
                    .iter()
                    .next()
                    .context("Could not get selected item")?
                    .output(),
            );
            return Ok(Some(next_args));
        }
        Event::EvActAccept(None) => {
            let _ = mode.run(&output);
            return Ok(None);
        }
        _ => {
            return Ok(None);
        }
    };
}

fn main() -> Result<()> {
    let mut cli = Cli::parse();
    loop {
        let mode = parse_mode(&cli.mode);
        if !mode.init(&cli.mode) {
            return Ok(());
        }
        match run_with(&mode, &cli) {
            Ok(Some(args)) => cli.update_from(
                [
                    vec![
                        std::env::args()
                            .next()
                            .context("Failed to get executable name")?
                            .as_str(),
                    ],
                    args.split(" ").collect(),
                ]
                .iter()
                .flatten(),
            ),
            Ok(None) => return Ok(()),
            Err(e) => return Err(e),
        }
    }
}
