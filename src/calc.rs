use std::{
    borrow::Cow,
    fs::File,
    io::{Read, Write},
    process::Command,
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
    fn set_options<'a>(
        &self,
        opts: &'a mut skim::prelude::SkimOptionsBuilder,
    ) -> &'a mut skim::prelude::SkimOptionsBuilder {
        opts.cmd(Some(format!(
            "{} calc --eval {}",
            std::env::args().next().unwrap(),
            "{}"
        )))
        .preview(Some(format!(
            "{} calc --eval {}",
            std::env::args().next().unwrap(),
            "{}"
        )))
        .show_cmd_error(true)
        .interactive(true)
        .bind(vec!["enter:accept(calc)".to_string()])
        .header(Some(format!(
            "calc - previous(_): {}",
            get_previous()
                .and_then(|x| Some(x.to_string()))
                .or(Some(String::from("N/A")))
                .unwrap()
        )))
    }
    fn run(&self, output: &SkimOutput) -> anyhow::Result<()> {
        let result = eval(&output.cmd);
        let _ = Command::new("wl-copy")
            .arg(&result.to_string())
            .spawn()
            .and_then(|mut h| h.wait());
        save_result(result);
        println!("{result}");
        Ok(())
    }
    fn init(&self, mode: &Mode) -> bool {
        match mode {
            Mode::Calc { eval: true, expr } => {
                println!("{}", eval(&expr.join(" ")));
                false
            }
            Mode::Calc { .. } => true,
            _ => true,
        }
    }
}

fn eval(expr: &str) -> f64 {
    let ctx = eva::lex::FunctionContext::default();
    let result: f64 = eva::eval_expr(&ctx, 12, expr, get_previous())
        .or_else(|e| -> Result<f64, ()> {
            panic!("Failed to evaluate {}: {}", expr, e);
        })
        .unwrap();
    return result;
}

fn get_previous() -> Option<f64> {
    File::open(PREV_RESULT_FILE)
        .and_then(|mut f| {
            let mut buf = String::new();
            let _ = f.read_to_string(&mut buf);
            return Ok(buf.parse::<f64>().unwrap_or_default());
        })
        .ok()
}
fn save_result(result: f64) {
    let f = File::create(PREV_RESULT_FILE);
    if f.is_ok() {
        let _ = f.unwrap().write_all(result.to_string().as_bytes());
    }
}
