use clap::{Parser, Subcommand};
use nix::{sys::signal::Signal, unistd::Pid};
use std::path::PathBuf;

#[derive(Parser)]
#[command(multicall = true)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Debug, Subcommand)]
pub enum Cmd {
    #[command(hide = true)]
    Ext {
        comm: String,
        #[arg(allow_hyphen_values = true)]
        args: Vec<String>,
    },
    Pwd,
    Echo {
        #[clap(default_value = "")]
        string: String,
    },
    Cd {
        path: PathBuf,
    },
    Exec {
        comm: String,
        #[arg(allow_hyphen_values = true)]
        args: Vec<String>,
    },
    Kill {
        signal: Signal,
        #[arg(value_parser = clap::value_parser!(i32))]
        pid: Pid,
    },
    Ps,
    Exit,
}
