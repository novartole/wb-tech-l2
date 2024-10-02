use crate::{
    cmd::{Cli, Cmd},
    eval::Expr,
    line::Line,
};
use anyhow::{anyhow, Error, Result};
use clap::Parser;

#[derive(Debug)]
pub enum Redir {
    Pipe,
    Fork,
}

#[derive(Debug)]
pub enum Token {
    Cmd(Cmd),
    Redir(Redir),
}

impl<'a> TryFrom<&Line<'a>> for Expr {
    type Error = Error;

    fn try_from(line: &Line<'a>) -> Result<Self, Self::Error> {
        let mut from = {
            let mut vec = Vec::try_from(line)?;
            vec.reverse();
            vec
        };
        let mut into = vec![];

        parse_in_place(&mut from, &mut into)?;

        Ok(into.pop().unwrap())
    }
}

impl<'a> TryFrom<&Line<'a>> for Vec<Token> {
    type Error = Error;

    fn try_from(line: &Line<'a>) -> Result<Self, Self::Error> {
        let mut tokens = vec![];
        let args = shlex::split(line).ok_or(anyhow!("command not found"))?;
        let mut buf = args.as_slice();

        while !buf.is_empty() {
            let cmd = {
                let cmd = {
                    let pos = buf
                        .iter()
                        .position(|arg| ["|", "&"].contains(&arg.as_str()))
                        .unwrap_or(buf.len());
                    let (cmd, suf) = buf.split_at(pos);
                    buf = suf;
                    cmd
                };
                Cli::try_parse_from(cmd).map(|cli| cli.cmd).unwrap_or({
                    let comm = cmd[0].to_owned();
                    let args = cmd[1..].iter().map(String::to_owned).collect();
                    Cmd::Ext { comm, args }
                })
            };

            tokens.push(Token::Cmd(cmd));

            match buf.split_first() {
                Some((redir, suf)) => {
                    match redir.as_str() {
                        "|" => tokens.push(Token::Redir(Redir::Pipe)),
                        "&" => tokens.push(Token::Redir(Redir::Fork)),
                        _ => {}
                    }
                    buf = suf;
                }
                None => break,
            }
        }

        Ok(tokens)
    }
}

fn parse_in_place(from: &mut Vec<Token>, into: &mut Vec<Expr>) -> Result<()> {
    loop {
        let token = match from.pop() {
            None => return Ok(()),
            Some(token) => token,
        };

        let expr = match token {
            Token::Cmd(cmd) => Expr::Cmd(cmd),
            Token::Redir(redir) => match redir {
                Redir::Pipe => {
                    let left = into
                        .pop()
                        .map(Box::new)
                        .ok_or(anyhow!("miss left pipe argument"))?;
                    let right = {
                        parse_in_place(from, into)?;
                        into.pop()
                            .map(Box::new)
                            .ok_or(anyhow!("miss right pipe argument"))?
                    };
                    Expr::Pipe { left, right }
                }
                Redir::Fork => {
                    let left = into
                        .pop()
                        .map(Box::new)
                        .ok_or(anyhow!("miss left pipe argument"))?;
                    let right = {
                        parse_in_place(from, into)?;
                        into.pop().map(Box::new)
                    };
                    Expr::Fork { left, right }
                }
            },
        };

        into.push(expr);
    }
}
