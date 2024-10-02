use crate::cmd::Cmd;
use anyhow::{anyhow, bail, Result};
use nix::sys::signal::kill;
use std::{
    borrow::Cow,
    env,
    io::{self, BufRead, BufReader, Write},
    iter::{once, repeat},
    path::{Path, PathBuf},
    process::{self, Command, Stdio},
};

#[derive(Debug)]
pub enum Expr {
    Cmd(Cmd),
    Pipe {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Fork {
        left: Box<Expr>,
        right: Option<Box<Expr>>,
    },
}

impl Expr {
    pub fn evaluate(self, args: Option<String>) -> Result<String> {
        let output = match self {
            Expr::Cmd(cmd) => match cmd {
                Cmd::Exec { comm, args: c_args } => {
                    let output = exec(comm, c_args, args)?;
                    println!("{}", output);
                    process::exit(0);
                }
                Cmd::Exit => process::exit(0),
                Cmd::Ext { comm, args: c_args } => exec(comm, c_args, args).map(Some)?,
                Cmd::Kill { signal, pid } => kill(pid, signal).map(|_| None)?,
                Cmd::Cd { path } => chdir(path).map(|_| None)?,
                Cmd::Echo { string } => Some(string),
                Cmd::Pwd => pwd().map(Some)?,
                Cmd::Ps => ps().map(Some)?,
            },
            Expr::Pipe { left, right } => {
                let mut output = left.evaluate(args)?;
                let args = Some(output);
                output = right.evaluate(args)?;
                output.into()
            }
            Expr::Fork { left, right } => {
                use nix::{
                    sys::wait::waitpid,
                    unistd::{fork, ForkResult},
                };

                match unsafe { fork() } {
                    Ok(ForkResult::Parent { child, .. }) => {
                        println!("  + {}", child);

                        std::thread::spawn(move || {
                            waitpid(child, None).unwrap();
                            println!("  + {} done", child);
                        });

                        match right {
                            Some(expr) => expr.evaluate(args).map(Some),
                            None => Ok(None),
                        }
                    }
                    Ok(ForkResult::Child) => {
                        let output = left.evaluate(args).map(Some)?;
                        println!("{}", output.unwrap_or_default());
                        process::exit(0);
                    }
                    Err(why) => bail!("fork failed: {}", why),
                }?
            }
        };

        Ok(output.unwrap_or_default())
    }
}

fn ps() -> Result<String> {
    let ps_output = Command::new("ps")
        .args(["-o pid=", "-o comm=", "-o etime="])
        .stdout(Stdio::piped())
        .spawn()?
        .stdout
        .ok_or_else(|| anyhow!("failed to capture STDIO"))?;

    let reader = BufReader::new(ps_output);

    Ok(reader
        .lines()
        .map_while(|try_line| {
            use std::fmt::Write;

            let mut line = try_line.ok()?;
            let time_pos = 1 + line.rfind(' ')?;
            let mls = {
                let etime = &line[time_pos..];
                let units = etime.split(['-', ':']).rev();
                once(100)
                    .chain(repeat(60))
                    .zip(units)
                    .try_fold(0, |mls, (mul, unit)| {
                        unit.parse::<u32>().map(|unit| mul * unit + mls).ok()
                    })?
            };
            line.truncate(time_pos);
            write!(&mut line, "{}", mls).ok()?;

            Some(line)
        })
        .fold(String::new(), |mut output, line| {
            output.push_str(&line);
            output.push('\n');
            output
        }))
}

fn exec(comm: String, args: Vec<String>, io_args: Option<String>) -> Result<String> {
    let mut prc = Command::new(comm)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    prc.stdin
        .as_mut()
        .unwrap()
        .write_all(io_args.unwrap_or_default().as_bytes())?;

    let output = prc.wait_with_output()?;

    if !output.status.success() {
        bail!("{}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn pwd() -> Result<String, io::Error> {
    env::current_dir().map(|cur_dir| format!("{}", cur_dir.display()))
}

fn chdir(path: impl AsRef<Path>) -> Result<()> {
    let target = match path.as_ref().to_str() {
        Some("~") => env::var("HOME").map(PathBuf::from).map(Cow::Owned)?,
        _ => path.as_ref().into(),
    };

    if let Err(e) = env::set_current_dir(path.as_ref()) {
        if let io::ErrorKind::NotFound = e.kind() {
            bail!("{}: no such file or directory", target.to_string_lossy());
        };

        return Err(anyhow!(e));
    }

    Ok(())
}
