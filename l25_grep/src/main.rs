mod cli;

use std::{
    borrow::Cow,
    collections::VecDeque,
    error::Error,
    fs,
    io::{self, BufRead},
    path::Path,
};

use clap::Parser;
use cli::Cli;
use regex::Regex;

fn main() -> Result<(), Box<dyn Error>> {
    let Cli {
        pattern,
        file,
        after_context,
        before_context,
        context,
        count,
        ignore_case,
        invert_match,
        fixed_string,
        line_number,
    } = Cli::parse();

    let context = if context > 0 {
        (context, context)
    } else {
        (before_context, after_context)
    };

    let total = for_each_match(
        AppRegex::build(pattern, ignore_case, fixed_string, invert_match, context)?,
        file,
        |(id, line)| {
            if count {
                return;
            }

            if line_number {
                println!("{}: {}", id, line);
            } else {
                println!("{}", line);
            }
        },
    )?;

    if count {
        println!("{}", total);
    }

    Ok(())
}

fn for_each_match(
    re: AppRegex,
    file: impl AsRef<Path>,
    mut f: impl FnMut((usize, String)),
) -> Result<usize, io::Error> {
    let reader = &mut {
        let file = fs::File::open(file)?;
        io::BufReader::new(file)
    };

    // exact matches
    let mut count = 0;
    // all we need in one place
    let mut handle_match = |(state, id, line)| {
        f((id, line));

        if let Match::Exact = state {
            count += 1;
        }
    };
    // keep matches and update state automatically
    let mut mtchs = Matches::new(re.before, re.after);

    // handle each line of file
    for (id, try_line) in reader.lines().enumerate() {
        let line = try_line?;
        let is_match = re.is_match(&line);

        // handle match which left buffer
        if let Some(val) = mtchs.insert(is_match, id, line) {
            handle_match(val);
        }
    }

    // don't forget to handle once done with file
    mtchs.clear().into_iter().for_each(handle_match);

    Ok(count)
}

struct Matches {
    buf: VecDeque<(Match, usize, String)>,
    before: usize,
    after: usize,
    exacts: VecDeque<usize>,
}

impl Matches {
    fn new(before: usize, after: usize) -> Self {
        let buf = VecDeque::with_capacity(before + 1 + after);
        let exacts = VecDeque::new();

        Self {
            buf,
            before,
            after,
            exacts,
        }
    }

    fn insert(&mut self, is_match: bool, id: usize, val: String) -> Option<(Match, usize, String)> {
        let buf = &mut self.buf;

        // if it's less, then just fill out buffer
        let res = if buf.len() < buf.capacity() {
            buf.push_back((Match::from(is_match), id, val));
            None
        // pop first - it will be returned,
        // and push a new match to the end
        } else {
            let res = buf.pop_front().and_then(|(state, id, line)| {
                if matches!(state, Match::Skip) {
                    None
                } else {
                    Some((state, id, line))
                }
            });

            buf.push_back((Match::from(is_match), id, val));

            res
        };

        // remark existed matches
        self.update();

        res
    }

    fn clear(&mut self) -> impl IntoIterator<Item = (Match, usize, String)> + '_ {
        self.buf
            .drain(..)
            .filter(|(state, ..)| !matches!(state, Match::Skip))
    }

    fn update(&mut self) {
        let buf = &mut self.buf;
        let last = buf.len() - 1;

        match buf[last] {
            (Match::Aside | Match::Skip, ..) => {
                'out: loop {
                    for &exact_id in &self.exacts {
                        let (_, first_buf_id, _) = buf[0];

                        if exact_id + self.after < first_buf_id {
                            self.exacts.pop_front();
                            continue 'out;
                        }

                        // fir
                        // [..  ex + af

                        // fir
                        // [..  ex .. ex + af .. last

                        let (left, right) = if exact_id <= first_buf_id {
                            (0, exact_id + self.after - first_buf_id + 1)
                        } else {
                            let left = exact_id - first_buf_id;
                            (left, buf.len().min(left + self.after))
                        };

                        (left..right).for_each(|id| {
                            if let (Match::Skip, ..) = buf[id] {
                                let (state, ..) = &mut buf[id];
                                *state = Match::Aside;
                            }
                        });
                    }
                    break;
                }
                return;
            }
            (_, id, _) => self.exacts.push_back(id),
        };

        (last.saturating_sub(self.before)..last).for_each(|id| {
            if let (Match::Skip, ..) = buf[id] {
                let (state, ..) = &mut buf[id];
                *state = Match::Aside;
            }
        });
    }
}

impl From<Match> for bool {
    fn from(value: Match) -> Self {
        matches!(value, Match::Exact | Match::Aside)
    }
}

struct AppRegex {
    inner: Regex,
    invert_match: bool,
    before: usize,
    after: usize,
}

impl AppRegex {
    fn build(
        pattern: impl AsRef<str>,
        ignore_case: bool,
        fixed_string: bool,
        invert_match: bool,
        (before_context, after_context): (u32, u32),
    ) -> Result<Self, regex::Error> {
        let hay = pattern.as_ref();
        let mut re = Cow::Borrowed(hay);

        if fixed_string {
            re = regex::escape(hay).into();
        }

        if ignore_case {
            re = format!("(?i){}", re).into();
        }

        let inner = Regex::new(&re)?;

        let before = before_context as usize;
        let after = after_context as usize;

        Ok(Self {
            inner,
            invert_match,
            before,
            after,
        })
    }

    fn is_match(&self, haystack: &str) -> bool {
        self.inner.is_match(haystack) ^ self.invert_match
    }
}

#[derive(Debug)]
pub enum Match {
    Skip,
    Exact,
    Aside,
}

impl From<bool> for Match {
    fn from(value: bool) -> Self {
        if value {
            Self::Exact
        } else {
            Self::Skip
        }
    }
}
