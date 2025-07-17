use crate::SkimRun;
#[cfg(feature = "apps")]
use crate::apps;
#[cfg(feature = "calc")]
use crate::calc;
#[cfg(feature = "hyprland")]
use crate::hyprctl_clients;
#[cfg(feature = "paru")]
use crate::paru;
#[cfg(feature = "systemd")]
use crate::systemd_services;

pub fn parse_mode(mode: &Mode) -> Box<dyn SkimRun> {
    use Mode::*;
    match mode {
        #[cfg(feature = "apps")]
        Apps {} => Box::new(apps::Apps),
        #[cfg(feature = "calc")]
        Calc { .. } => Box::new(calc::Calc),
        #[cfg(feature = "hyprland")]
        HyprctlClients {} => Box::new(hyprctl_clients::HyprctlClients),
        #[cfg(feature = "systemd")]
        SystemdServices {} => Box::new(systemd_services::SystemdServices),
        #[cfg(feature = "paru")]
        Paru {} => Box::new(paru::Paru),
        #[allow(unreachable_patterns)]
        _ => panic!("This mode is not enabled in this build. Enable the corresponding feature."),
    }
}

#[derive(Debug, clap::Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub mode: Mode,
    #[arg(short, long, global = true)]
    pub query: Option<String>,
    #[arg(short, long, global = true, value_delimiter = ',')]
    pub modes: Vec<String>,
}
#[derive(Debug, Clone, clap::Subcommand)]
pub enum Mode {
    #[cfg(feature = "apps")]
    Apps {},
    #[cfg(feature = "calc")]
    Calc {
        #[arg(long, default_value = "false")]
        eval: bool,
        expr: Vec<String>,
    },
    #[cfg(feature = "hyprland")]
    HyprctlClients {},
    #[cfg(feature = "systemd")]
    SystemdServices {},
    #[cfg(feature = "paru")]
    Paru {},
}
impl ToString for Mode {
    fn to_string(&self) -> String {
        use Mode::*;
        match self {
            #[cfg(feature = "apps")]
            Apps { .. } => "apps".to_string(),
            #[cfg(feature = "calc")]
            Calc { .. } => "calc".to_string(),
            #[cfg(feature = "hyprland")]
            HyprctlClients { .. } => "hyprctl-clients".to_string(),
            #[cfg(feature = "systemd")]
            SystemdServices { .. } => "systemd-services".to_string(),
            #[cfg(feature = "paru")]
            Paru { .. } => "paru".to_string(),
        }
    }
}
