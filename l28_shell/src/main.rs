mod cmd;
mod eval;
mod line;
mod parser;

use anyhow::{anyhow, Result};
use std::io;

use crate::{eval::Expr, line::Line};

fn main() {
    let mut buf = String::new();

    loop {
        if let Err(why) = repl(&mut buf) {
            println!("error: {}", why);
        }
    }
}

fn repl(buf: &mut String) -> Result<()> {
    let line = &read_line(buf)?;

    if line.is_empty() {
        return Ok(());
    }

    let output = Expr::try_from(line)
        .map_err(|_| anyhow!("command not found"))?
        .evaluate(None)?;

    println!("{}", output);

    Ok(())
}

fn read_line(buf: &mut String) -> Result<Line<'_>> {
    io::stdin().read_line(buf)?;
    let line = Line::drain(buf);

    Ok(line)
}
