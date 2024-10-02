use std::{
    fmt, fs,
    io::{self, BufRead},
    path::{Path, PathBuf},
};

use clap::Parser;

type Counts = (Option<usize>, Option<usize>, Option<usize>);

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("failed to proceed: {0}")]
    Io(#[from] io::Error),

    #[error("failed to print result: {0}")]
    Fmt(#[from] fmt::Error),
}

#[derive(Parser)]
struct Cli {
    /// файл
    pub file: PathBuf,

    /// показать количество символов в файле
    #[clap(short, default_value_t = false)]
    pub chars: bool,

    /// вывести количество строк в файле
    #[clap(short, default_value_t = false)]
    pub lines: bool,

    /// отобразить количество слов в файле
    #[clap(
        short,
        default_value_t = true,
        default_value_ifs = [
            ("chars", "true", "false"),
            ("lines", "true", "false"),
        ]
    )]
    pub words: bool,
}

fn main() -> Result<(), Error> {
    let Cli {
        file,
        chars,
        lines,
        words,
    } = Cli::parse();

    // get counts according to parameters
    let counts = count(&file, chars, lines, words)?;

    // prepare output string
    let output = format_with(file, counts)?;

    // print out result
    println!("{}", output);

    Ok(())
}

/// Calculate count of _chars_/_lines_/_words_ of _file_ based on input flag(s).
///
/// # Error
/// It might fail with _io::Error_ if an error occurred while reading _file_.
fn count(
    file: impl AsRef<Path>,
    chars: bool,
    lines: bool,
    words: bool,
) -> Result<Counts, io::Error> {
    let file = fs::File::open(file)?;

    let mut chars = if chars { Some(0) } else { None };
    let mut lines = if lines { Some(0) } else { None };
    let mut words = if words { Some(0) } else { None };

    for try_line in io::BufReader::new(file).lines() {
        let line = try_line?;

        if let Some(chars) = chars.as_mut() {
            // \n should also be counted, so +1
            *chars += 1 + line.chars().count();
        }

        if let Some(lines) = lines.as_mut() {
            *lines += 1;
        }

        if let Some(words) = words.as_mut() {
            *words += line
                // separate words by whitespace
                .split_whitespace()
                // empty strings aren't words
                .filter(|str| !str.is_empty())
                .count();
        }
    }

    Ok((chars, lines, words))
}

/// Prepare (format) result.
///
/// # Error
/// It might fail if for some reason it cannot write into local buffer,
/// while preparing output result.
fn format_with(
    filename: impl AsRef<Path>,
    (chars, lines, words): Counts,
) -> Result<String, fmt::Error> {
    // needs for write! macro
    use std::fmt::Write as _;

    let mut buf = String::new();

    if let Some(chars) = chars {
        write!(&mut buf, "{}", chars)?;
    }

    if let Some(lines) = lines {
        write!(
            &mut buf,
            "{}{}",
            if chars.is_some() { " " } else { "" },
            lines
        )?;
    }

    if let Some(words) = words {
        write!(
            &mut buf,
            "{}{}",
            if chars.is_some() || lines.is_some() {
                " "
            } else {
                ""
            },
            words
        )?;
    }

    write!(&mut buf, " {:?}", filename.as_ref().display())?;

    Ok(buf)
}
