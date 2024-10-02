mod cli;

use std::{borrow::Cow, collections::HashSet, io, mem};

use cli::{Cli, Parser};

fn main() {
    // parse and prepare arguments
    let (flds, del, sep) = parse_args();

    // buffer to read into
    let mut in_buf = String::new();

    // read and process each line on the fly
    // until no input or an error occurrs
    while 0 < io::stdin()
        .read_line(&mut in_buf)
        .expect("failed to read line")
    {
        // get line and clear buffer
        let buf = in_buf.drain(..);
        // \n is de trop in the end
        let line = buf.as_str().trim_end();

        // maybe yield some matched columns
        let mb_cols = cut(line, &flds, del);

        // skip if no matches but _separated_
        if mb_cols.is_none() && sep {
            continue;
        }

        // print out formatted result string
        println!("{}", format_with_or(line, mb_cols, del));
    }
}

fn parse_args() -> (HashSet<usize>, char, bool) {
    let Cli {
        fields,
        delimiter,
        separated,
    } = Cli::parse();

    // keep only unique fields (columns)
    let columns = fields.into_iter().map(|val| val as _).collect();

    (columns, delimiter, separated)
}

/// Separate `line` by appropriate delimiter ([`del`]) and return columns according to fields ([`flds`]).
///
/// # Schema
/// |------|-----|-----------|
/// | -del |  _  | None      |
/// | +del | +id | Some(col) |
/// | +del | -id | None      |
/// |------|-----|-----------|
///
/// # Legend
/// - ±`del` - line does (not) have delimiter,
/// - ±`id` - fields do (not) contain column id,
/// - 3rd column - value of item of iterator.
fn cut<'a>(
    line: &'a str,
    flds: &'a HashSet<usize>,
    del: char,
) -> Option<impl Iterator<Item = Option<&'a str>>> {
    // no delimiter - no talk
    if !line.contains(del) {
        return None;
    }

    // get columns according to Schema
    let cols = line
        .split(del)
        .enumerate()
        .map(|(id, col)| match flds.contains(&id) {
            true => Some(col),
            false => None,
        });

    Some(cols)
}

/// Build formatted string from iterator (`mb_cols`) over columns.
/// When iterator is None, return owned default value (`dflt`).
fn format_with_or<'a>(
    dflt: &'a str,
    mb_cols: Option<impl Iterator<Item = Option<&'a str>>>,
    del: char,
) -> Cow<'a, str> {
    match mb_cols {
        None => dflt.into(),
        Some(cols) => {
            // start with Cow to avoid alloc if no cols
            let mut fmtd: Cow<'_, str> = Cow::default();
            // true if there is a col
            let mut any = false;

            // join cols with del if any
            for col in cols.flatten() {
                let buf = fmtd.to_mut();

                if mem::replace(&mut any, true) {
                    buf.push(del);
                }

                buf.push_str(col);
            }

            fmtd
        }
    }
}
