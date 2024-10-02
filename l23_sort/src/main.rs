mod cli;
use clap::Parser;
use cli::{Cli, SortType};
use std::{
    cmp::Reverse,
    fs::File,
    io::{self, BufRead, BufReader},
};

fn main() {
    let cli = Cli::parse();

    let mut lines = BufReader::new(File::open(&cli.file).expect("[err]: opening file"))
        .lines()
        .enumerate()
        .try_fold(vec![], |mut lines, (i, try_line)| {
            lines.push((Some(i), try_line?));
            Ok::<_, io::Error>(lines)
        })
        .expect("[err]: reading line");

    let mut keys = lines
        .iter()
        .try_fold(vec![], |mut keys, (line_id, line)| {
            let key = {
                let mut itr = line.split(" ");
                let key_id = cli.key as usize - 1;

                let key = if cli.ignore_leading_blanks {
                    itr.filter(|str| !str.is_empty()).nth(key_id)
                } else {
                    itr.nth(key_id)
                }
                .map(|str| SortKey::new(cli.sort, str))
                .unwrap_or(SortKey::from(line));

                (key, Reverse(line_id))
            };

            keys.push(key);

            Ok::<_, io::Error>(keys)
        })
        .expect("[err]: reading file");

    keys.sort();

    if cli.unique {
        keys.dedup_by_key(|(key, _)| *key);
    }

    if cli.reverse {
        keys.reverse();
    }

    if cli.check
        && lines
            .iter()
            .zip(keys.iter())
            .any(|((a, _), (_, Reverse(b)))| a != *b)
    {
        return println!("disordered");
    }

    let ids: Vec<_> = keys
        .iter()
        .map(|(_, Reverse(id))| **id)
        .chain(std::iter::repeat(None))
        .enumerate()
        .take(lines.len())
        .collect();

    drop(keys);

    lines.iter_mut().for_each(|(id, _)| *id = None);

    for (a, b) in ids {
        if let Some(id) = b {
            lines[id].0 = Some(a);
        }
    }

    lines.sort_by_key(|(id, _)| *id);

    for line in lines.iter().filter(|(maybe_id, _)| maybe_id.is_some()) {
        println!("{:?}", line);
    }
}

#[derive(Clone, Copy, Debug)]
enum Month {
    Jan,
    Feb,
    Mar,
    Apr,
    May,
    Jun,
    Jul,
    Aug,
    Sep,
    Oct,
    Nov,
    Dec,
}

impl Month {
    fn from(str: &str) -> Option<Self> {
        use Month::*;

        Some(match str {
            "jan" => Jan,
            "feb" => Feb,
            "mar" => Mar,
            "apr" => Apr,
            "may" => May,
            "jun" => Jun,
            "jul" => Jul,
            "aug" => Aug,
            "sep" => Sep,
            "oct" => Oct,
            "nov" => Nov,
            "dec" => Dec,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug)]
enum SortKeyType {
    String,
    Undefined,
    Numeric(f64),
    Month(Month),
    HumanNumeric(f64),
}

impl Default for SortKeyType {
    fn default() -> Self {
        Self::Undefined
    }
}

#[derive(Clone, Copy, Debug)]
struct SortKey<'a> {
    inner: &'a str,
    r#type: SortKeyType,
}

impl Eq for SortKey<'_> {}

impl Ord for SortKey<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (&self.r#type, &other.r#type) {
            // Numeric vs. Numeric
            // or HumanNumeric vs. HumanNumeric
            (SortKeyType::Numeric(a), SortKeyType::Numeric(b))
            | (SortKeyType::HumanNumeric(a), SortKeyType::HumanNumeric(b)) => a.total_cmp(b),
            // Month vs. Month
            (SortKeyType::Month(a), SortKeyType::Month(b)) => (*a as usize).cmp(&(*b as usize)),
            // compare others by source string
            _ => self.inner.cmp(other.inner),
        }
    }
}

impl PartialEq for SortKey<'_> {
    fn eq(&self, other: &Self) -> bool {
        matches!(self.cmp(other), std::cmp::Ordering::Equal)
    }
}

impl PartialOrd for SortKey<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> SortKey<'a> {
    fn new(sort_type: SortType, inner: &'a str) -> Self {
        use human_format::{Formatter, Scales};

        let r#type = match sort_type {
            SortType::String => SortKeyType::String,
            SortType::Month => Month::from(inner)
                .map(SortKeyType::Month)
                .unwrap_or_default(),
            SortType::Numeric => inner
                .parse()
                .ok()
                .map(SortKeyType::Numeric)
                .unwrap_or_default(),
            SortType::Human => Formatter::new()
                .with_separator("")
                .with_scales(Scales::Binary())
                .try_parse(inner)
                .ok()
                .map(SortKeyType::HumanNumeric)
                .unwrap_or_default(),
        };

        Self { inner, r#type }
    }
}

impl<'a> From<&'a String> for SortKey<'a> {
    fn from(inner: &'a String) -> Self {
        let r#type = SortKeyType::default();

        Self { inner, r#type }
    }
}
