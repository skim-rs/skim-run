use crate::{SkimRun, apps, calc, hyprctl_clients, systemd_services};

pub fn parse_mode(mode: &Mode) -> Box<dyn SkimRun> {
    use Mode::*;
    match mode {
        Apps {} => Box::new(apps::Apps),
        Calc { .. } => Box::new(calc::Calc),
        HyprctlClients {} => Box::new(hyprctl_clients::HyprctlClients),
        SystemdServices {} => Box::new(systemd_services::SystemdServices),
    }
}

#[derive(Debug, clap::Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub mode: Mode,
    #[arg(short, long, global = true)]
    pub query: Option<String>,
    #[arg(short, long, global = true)]
    pub modes: Vec<String>,
}
#[derive(Debug, Clone, clap::Subcommand)]
pub enum Mode {
    Apps {},
    Calc {
        #[arg(long, default_value = "false")]
        eval: bool,
        expr: Vec<String>,
    },
    HyprctlClients {},
    SystemdServices {},
}
