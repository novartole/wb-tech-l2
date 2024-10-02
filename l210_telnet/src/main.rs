use std::{
    error::Error,
    net::SocketAddr,
    str::FromStr,
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use clap::Parser;
use duration_string::DurationString;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{lookup_host, TcpStream},
    select,
};

#[tokio::main]
async fn main() {
    App::from(parse_args().await.expect("[err]: parsing arguments"))
        .run()
        .await;
    std::process::exit(0);
}

#[derive(Parser)]
pub struct Cli {
    /// хост
    pub host: String,

    /// порт
    #[arg(
        value_parser = clap::value_parser!(u16).range(1..),
        default_value_t = 23,
    )]
    pub port: u16,

    /// таймаут на подключение к серверу
    #[clap(
        long,
        default_value = "10s", 
        value_parser = timeout_parser,
    )]
    pub timeout: Duration,
}

fn timeout_parser(arg: &str) -> Result<Duration, duration_string::Error> {
    DurationString::from_str(arg).map(Into::into)
}

async fn parse_args() -> Result<(SocketAddr, Duration), Box<dyn Error>> {
    let Cli {
        host,
        port,
        timeout,
    } = Cli::parse();

    let addr = lookup_host(format!("{}:{}", host, port))
        .await?
        .next()
        .ok_or(format!(r#"[err]: failed to resolve "{}:{}""#, host, port))?;

    Ok((addr, timeout))
}

struct App {
    addr: SocketAddr,
    timeout: Duration,
}

impl From<(SocketAddr, Duration)> for App {
    fn from((addr, timeout): (SocketAddr, Duration)) -> Self {
        Self { addr, timeout }
    }
}

impl App {
    async fn run(self) {
        run_with(self.addr, self.timeout).await
    }
}

async fn run_with(addr: SocketAddr, timeout: Duration) {
    let mut stream = select! {
        try_stream = TcpStream::connect(addr) => {
            match try_stream {
                Err(e) => return println!("[err]: failed establishing connection to {}: {}", addr, e),
                Ok(stream) => stream
            }
        },
        _ = tokio::time::sleep(timeout) => {
            return println!("[err]: timeout expired while connecting to {}", addr);
        },
    };

    let (mut socket_lines, mut socket_writer) = {
        let (reader, writer) = stream.split();
        (BufReader::new(reader).lines(), writer)
    };
    let mut stdin_lines = BufReader::new(tokio::io::stdin()).lines();
    let written: Arc<AtomicBool> = Default::default();

    let read_from_socket = {
        let written_ = Arc::clone(&written);

        async move {
            loop {
                select! {
                    res = socket_lines.next_line() => {
                        match res {
                            Err(e) => break println!("[err]: failed reading from socket: {}", e),
                            Ok(Some(line)) => println!("{}", line),
                            _ => break,
                        }
                    },
                    _ = tokio::time::sleep(timeout) => {
                        if written_.load(std::sync::atomic::Ordering::SeqCst) {
                            break println!("[err]: timeout expired while reading from socket");
                        }
                    }
                }
            }
        }
    };

    let write_into_socket = async move {
        loop {
            let line = match stdin_lines.next_line().await {
                Err(e) => break println!("[err]: failed reading from socket: {}", e),
                Ok(Some(line)) => line,
                _ => break,
            };

            select! {
                res = socket_writer.write_all(line.as_bytes()) => {
                    if let Err(e) = res {
                        break println!("[err]: failed writting to socket: {}", e);
                    }
                    if let Err(e) = socket_writer.flush().await {
                        break println!("[err]: failed flushing output stream: {}", e);
                    }
                    written.store(true, std::sync::atomic::Ordering::SeqCst);
                },
                _ = tokio::time::sleep(timeout) => {
                    break println!("[err]: timeout expired while writting from socket");
                },
            }
        }
    };

    select! {
        _ = read_from_socket => {},
        _ = write_into_socket => {},
    };
}
