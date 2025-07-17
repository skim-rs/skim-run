use crate::SkimRun;
#[cfg(feature = "apps")]
use crate::apps;
#[cfg(feature = "calc")]
use crate::calc;
#[cfg(feature = "hyprland")]
use crate::hyprctl_clients;
#[cfg(feature = "hyprland")]
use crate::hyprctl_hide;
#[cfg(feature = "paru")]
use crate::paru;
#[cfg(feature = "systemd")]
use crate::systemd_services;

/// Parses the CLI mode and returns the corresponding `SkimRun` implementation.
///
/// # Panics
/// Panics if the mode is not enabled in this build.
///
/// # Errors
/// This function does not return errors, but the returned `SkimRun` may fail at runtime.
#[must_use]
pub fn parse_mode(mode: &Mode) -> Box<dyn SkimRun> {
    #[cfg(feature = "apps")]
    use Mode::Apps;
    #[cfg(feature = "calc")]
    use Mode::Calc;
    #[cfg(feature = "hyprland")]
    use Mode::{HyprctlClients, HyprctlHide};
    #[cfg(feature = "systemd")]
    use Mode::SystemdServices;
    #[cfg(feature = "paru")]
    use Mode::Paru;

    match mode {
        #[cfg(feature = "apps")]
        Apps {} => Box::new(apps::Apps),
        #[cfg(feature = "calc")]
        Calc { .. } => Box::new(calc::Calc),
        #[cfg(feature = "hyprland")]
        HyprctlClients {} => Box::new(hyprctl_clients::HyprctlClients),
        #[cfg(feature = "hyprland")]
        HyprctlHide { ignore_class, swap } => Box::new(hyprctl_hide::HyprctlHide { ignore_class: ignore_class.clone(), swap: swap.clone() }),
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
    #[cfg(feature = "hyprland")]
    HyprctlHide {
        #[arg(long)]
        ignore_class: Option<String>,
        #[arg(long)]
        swap: Option<String>,
    },
    #[cfg(feature = "systemd")]
    SystemdServices {},
    #[cfg(feature = "paru")]
    Paru {},
}
impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "apps")]
        use Mode::Apps;
        #[cfg(feature = "calc")]
        use Mode::Calc;
        #[cfg(feature = "hyprland")]
        use Mode::{HyprctlClients, HyprctlHide};
        #[cfg(feature = "systemd")]
        use Mode::SystemdServices;
        #[cfg(feature = "paru")]
        use Mode::Paru;

        let s = match self {
            #[cfg(feature = "apps")]
            Apps { .. } => "apps",
            #[cfg(feature = "calc")]
            Calc { .. } => "calc",
            #[cfg(feature = "hyprland")]
            HyprctlClients { .. } => "hyprctl-clients",
            #[cfg(feature = "hyprland")]
            HyprctlHide { .. } => "hyprctl-hide",
            #[cfg(feature = "systemd")]
            SystemdServices { .. } => "systemd-services",
            #[cfg(feature = "paru")]
            Paru { .. } => "paru",
        };
        write!(f, "{s}")
    }
}
