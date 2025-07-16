use anyhow::{Context as _, Result};
use clap::Parser;
use skim::prelude::*;
use skim_run::*;

fn run_with(mode: &Box<dyn SkimRun>, args: &Cli) -> Result<Option<String>> {
    let mut env_options = vec!["sk".to_string()];
    env_options.extend(
        std::env::var("SKIM_DEFAULT_OPTIONS")
            .ok()
            .and_then(|val| shlex::split(&val))
            .unwrap_or_default(),
    );
    let mut options = SkimOptions::parse_from(env_options);

    let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();

    mode.set_options(&mut options);

    options.query = args.query.clone();
    if !args.modes.is_empty() {
        let current_mode_idx = args
            .modes
            .iter()
            .position(|m| m.to_lowercase() == args.mode.to_string().to_lowercase())
            .context("Current mode not found in enabled modes")?;

        let next_mode = args.modes[(current_mode_idx + 1) % args.modes.len()].clone();
        options
            .bind
            .extend(vec![format!("tab:accept({})", next_mode)]);
        if let Some(h) = options.header {
            options.header = Some(format!("{} --- next mode(tab): {}", h, next_mode));
        } else {
            options.header = Some(format!("next mode(tab): {}", next_mode));
        }
    }
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
            let mut next_args = next_args.replace("{q}", &output.query);
            next_args = next_args.replace("{cq}", &output.cmd);
            if let Some(selected) = output.selected_items.iter().next() {
                next_args = next_args.replace("{}", &selected.output());
            }
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
