use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{self, BufRead},
    path::{Path, PathBuf},
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use clap::Parser;
use serde::Serialize;

#[rustfmt::skip]
const ASCII_ABC: [char; 52] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
    'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
    'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

#[derive(Parser)]
pub struct Cli {
    /// входной файл
    pub file: PathBuf,

    /// количество потоков, которое будет задействовано для подсчета
    #[clap(
        short, 
        default_value_t = 1, 
        value_parser = clap::value_parser!(u8).range(1..)
    )]
    pub threads: u8,
}

#[derive(Serialize)]
struct Output {
    #[serde(serialize_with = "se_duration")]
    elapsed: Duration,
    #[serde(flatten)]
    result: HashMap<char, usize>,
}

fn se_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let secs = format!("{} s", duration.as_secs_f32());

    serializer.serialize_str(&secs)
}

impl Output {
    fn from<T>(counts: T, elapsed: Duration) -> Self
    where
        T: IntoIterator<Item = (char, usize)>,
    {
        let result = counts.into_iter().collect();

        Self { result, elapsed }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // parse and prepare arguments
    let (file, threads) = parse_args();

    // get counts of ASCII letters
    // and measure taken time
    let (counts, taken_time) = {
        let now = Instant::now();
        let counts = count_ascii_letters(file, threads)?;
        let taken = now.elapsed();
        (counts, taken)
    };

    // create output model and serialize it into string
    let json = {
        let output = Output::from(counts, taken_time);
        serde_json::to_string(&output)?
    };

    // print result as json
    Ok(println!("{}", json))
}

fn parse_args() -> (impl AsRef<Path>, usize) {
    let Cli { file, threads } = Cli::parse();

    (file, threads as usize)
}

/// Implemeter can count any type of letters (and chars).
/// For instance, implementation for Unicode could look like this:
/// ```
/// impl CountLetters<std::collection::HashMap<char, usize>> for &str {
///     fn count_letters(&self, counts: &mut T) {
///         todo!()
///     }
/// }
/// ```
trait CountLetters<T>
where
    T: IntoIterator<Item = (char, usize)>,
{
    fn count_letters(&self, counts: &mut T);
}

impl CountLetters<[(char, usize); 52]> for &str {
    /// Count ASCII letters. Expected `counts` to be sorted by chars [[A-Za-z]].
    fn count_letters(&self, counts: &mut [(char, usize); 52]) {
        for ch in self.chars().filter(char::is_ascii_alphabetic) {
            // 6 chars gap between 'Z' and 'a' leads to offset
            let offset = if ch.is_ascii_uppercase() { 0 } else { 6 };
            // ASCII can safely be represented as u8.
            let id = (ch as u8 - b'A' - offset) as usize;
            // take mut count from container
            let (_, count) = &mut counts[id];

            *count += 1;
        }
    }
}

fn count_ascii_letters(
    file: impl AsRef<Path>,
    thrds: usize,
) -> Result<[(char, usize); 52], io::Error> {
    let file = File::open(file)?;

    let mut res = ASCII_ABC.map(|ch| (ch, 0));

    match thrds {
        // unreachable due to argument restriction
        0 => unreachable!(),
        // proceed file on current thread
        1 => {
            for try_line in io::BufReader::new(file).lines() {
                try_line?.as_str().count_letters(&mut res);
            }
        }
        // Start n worker threads (workers).
        // Each worker proceeds recieved line.
        // Return accumulated result once all workers finished.
        n => {
            // Main thread is Sender. Each worker gets same Receiver.
            // This approach allows to take out a line 
            // from Receier and unlock it for other workers.
            let (tx, rx) = {
                let (tx, rx) = mpsc::channel::<String>();
                (tx, Arc::new(Mutex::new(rx)))
            };

            // known count of workers - init with cap
            let mut wrkrs = Vec::with_capacity(n);

            for _ in 0..n {
                // keep handlers to sync once done
                let wrkr = thread::spawn({
                    let rx_ = Arc::clone(&rx);
                    // note: copy by value
                    let mut w_res = res;

                    move || loop {
                        // free lock immediately to give a chance to other workers
                        let mb_line = {
                            let lock = rx_.lock().unwrap();
                            lock.iter().next()
                        };

                        if let Some(line) = mb_line.as_deref() {
                            line.count_letters(&mut w_res);
                            continue;
                        }

                        break w_res;
                    }
                });

                wrkrs.push(wrkr);
            }

            // read and send line to workers
            for try_line in io::BufReader::new(file).lines() {
                tx.send(try_line?).unwrap();
            }

            // dropping single Sender finishes worker tasks
            drop(tx);

            // n x 52 = O(n) complexity even there is a nested loop
            for wrkr in wrkrs {
                for (pos, w_count) in wrkr
                    .join()
                    .unwrap()
                    .map(|(_, count)| count)
                    .into_iter()
                    .enumerate()
                {
                    let (_, count) = &mut res[pos];

                    *count += w_count;
                }
            }
        }
    }

    Ok(res)
}
