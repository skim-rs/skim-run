use std::{
    borrow::Cow,
    fs::File,
    io::{Read, Write},
    process::Command,
};

use rink_core::{
    CURRENCY_FILE, DATES_FILE, DEFAULT_FILE, ast, loader::gnu_units, output::QueryReply,
    parsing::datetime,
};
use skim::{SkimItem, SkimOutput};

use crate::{Mode, SkimRun};

static PREV_RESULT_FILE: &str = "/tmp/calc.prev";

#[derive(Default, Clone, Copy)]
pub struct Calc;

impl SkimItem for Calc {
    fn text(&self) -> Cow<str> {
        Cow::default()
    }
}

impl SkimRun for Calc {
    fn set_options(&self, opts: &mut skim::prelude::SkimOptions) {
        let exe = std::env::current_exe()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "skim-run".to_string());
        opts.cmd = Some(format!("{} calc --eval {}", exe, "'{}'"));
        opts.show_cmd_error = true;
        opts.interactive = true;
        opts.bind.extend(vec!["enter:accept(calc)".to_string()]);
        opts.header = Some(format!(
            "calc - previous(_): {}",
            get_previous().map_or(String::from("N/A"), |x| x.to_string())
        ));
    }
    fn run(&self, output: &SkimOutput) -> anyhow::Result<()> {
        let result = eval(&output.cmd);
        let _ = Command::new("wl-copy")
            .arg(format_result(&result))
            .spawn()
            .and_then(|mut h| h.wait());
        println!("{}", format_result(&result));
        save_result(&result);
        Ok(())
    }
    fn init(&self, mode: &Mode) -> bool {
        match mode {
            Mode::Calc { eval: true, expr } => {
                println!("{}", eval(&expr.join(" ")));
                false
            }
            _ => true,
        }
    }
}

fn eval(expr: &str) -> QueryReply {
    let mut ctx = rink_core::Context::new();

    if let Some(f) = DATES_FILE {
        ctx.load_dates(datetime::parse_datefile(f));
    }

    let mut currency_defs = Vec::new();
    match reqwest::blocking::get("https://rinkcalc.app/data/currency.json") {
        Ok(response) => match response.json::<ast::Defs>() {
            Ok(mut live_defs) => {
                currency_defs.append(&mut live_defs.defs);
            }
            Err(why) => println!("Error parsing currency json: {why}"),
        },
        Err(why) => println!("Error fetching up-to-date currency conversions: {why}"),
    }
    if let Some(f) = CURRENCY_FILE {
        currency_defs.append(&mut gnu_units::parse_str(f).defs);
    }
    let _ = ctx.load(ast::Defs {
        defs: currency_defs,
    });

    if let Some(f) = DEFAULT_FILE {
        let _ = ctx.load(gnu_units::parse_str(f));
    }

    ctx.set_time(chrono::Local::now());

    let mut expr = String::from(expr);
    if let Some(p) = get_previous() {
        expr = expr.replace('_', &p);
    }
    rink_core::eval(&mut ctx, &expr)
        .map_err(|e| {
            println!("Failed to evaluate {expr}: {e}");
            panic!();
        })
        .unwrap()
}

fn get_previous() -> Option<String> {
    File::open(PREV_RESULT_FILE)
        .map(|mut f| {
            let mut buf = String::new();
            let _ = f.read_to_string(&mut buf);
            buf
        })
        .ok()
}
fn save_result(result: &QueryReply) {
    if let Ok(mut file) = File::create(PREV_RESULT_FILE) {
        let _ = file.write_all(format_result(result).as_bytes());
    }
}
fn format_result(result: &QueryReply) -> String {
    let out = result.to_string();
    out.split(" (").next().unwrap_or_default().to_string()
}
