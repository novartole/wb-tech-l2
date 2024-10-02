fn main() {
    let string = parse_args();
    let result = unpack(&string).unwrap_or_else(|| panic!("failed to unpack {}", string));
    println!(r#""{}" => "{}""#, string, result);
}

/// Parse arguments.
///
/// # Panic
/// Expected one string argument to be provided,
/// panic if it gets nothing instead.
fn parse_args() -> String {
    let mut args = std::env::args();
    let prc = args.next().unwrap();
    args.next()
        .unwrap_or_else(|| panic!(r#"expected string: {} "some string""#, prc))
}

/// Unpack string according to rules:
/// - letter [L] followed by ASCII digit [N] gives [N] times [L],
/// - escaped digit considered to be a symbol, not a digit.
///
/// # Examples:
/// ```
/// "a4bc2d5e" => "aaaabccddddde"
/// "abcd" => "abcd"
/// "45" => "" (некорректная строка)
/// "" => ""
/// ```
fn unpack(str: &str) -> Option<String> {
    use Char::*;

    let mut buf = String::new();
    let chars = str.chars();
    let mut prev = None;

    for cur in chars {
        match prev.take() {
            Some(ch) => match ch {
                Raw('\\') if cur == '\\' => prev = Some(Escaped('\\')),
                Raw('\\') if cur.is_ascii_digit() => prev = Some(Escaped(cur)),
                Raw(ch) | Escaped(ch) => match cur.to_digit(10) {
                    Some(n) => {
                        for _ in 0..n.saturating_sub(1) {
                            buf.push(ch);
                        }
                        prev = Some(Raw(ch));
                    }
                    None => {
                        buf.push(ch);
                        prev = Some(Raw(cur));
                    }
                },
            },
            None if cur.is_ascii_digit() => return None,
            None => prev = Some(Raw(cur)),
        }
    }

    prev.inspect(|(Raw(ch) | Escaped(ch))| buf.push(*ch));

    Some(buf)
}

enum Char {
    Raw(char),
    Escaped(char),
}

/// It might be used to unpack ASCII.
/// It was written first, before knowing of Unicode restriction.
#[allow(dead_code)]
fn unpack_ascii(mut str: &str) -> Result<String, Box<dyn std::error::Error>> {
    use std::fmt::Write as _;

    let mut result = String::new();

    loop {
        match str.find(|ch: char| ch.is_ascii_digit()) {
            None => break Ok(result + str),
            Some(0) => break Err("expected chars followed by number".into()),
            Some(nd_end /* >= 1 */) => {
                result += &str[..nd_end - 1];

                let d_end = str[nd_end..]
                    .find(|ch: char| !ch.is_ascii_digit())
                    .map_or(str.len(), |val| nd_end + val);

                for _ in 0..str[nd_end..d_end].parse()? {
                    write!(&mut result, "{}", &str[nd_end - 1..nd_end])?;
                }

                str = &str[d_end..];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::unpack;

    #[test]
    fn empty() {
        assert_eq!(unpack("").unwrap(), "");
    }

    #[test]
    fn only_letters() {
        assert_eq!(unpack("abc").unwrap(), "abc");
    }

    #[test]
    fn start_with_number() {
        assert!(unpack("1abc").is_none());
        assert!(unpack("45").is_none());
    }

    #[test]
    fn strings_and_numbers() {
        assert_eq!(unpack("a4bc2d5e").unwrap(), "aaaabccddddde");
        assert_eq!(unpack("a1234").unwrap(), "aaaaaaa");
    }

    #[test]
    fn escaped() {
        assert_eq!(unpack(r"qwe\4\5").unwrap(), "qwe45");
        assert_eq!(unpack(r"qwe\1\2\3\").unwrap(), r"qwe123\");
        assert_eq!(unpack(r"qwe\1\2\3r").unwrap(), "qwe123r");

        assert_eq!(unpack(r"qwe\45").unwrap(), "qwe44444");
        assert_eq!(unpack(r"qwe\45\").unwrap(), r"qwe44444\");
        assert_eq!(unpack(r"qwe\45r").unwrap(), "qwe44444r");

        assert_eq!(unpack(r"qwe\\5").unwrap(), r"qwe\\\\\");
        assert_eq!(unpack(r"qwe\\5\\").unwrap(), r"qwe\\\\\\");
        assert_eq!(unpack(r"qwe\\5r").unwrap(), r"qwe\\\\\r");
        assert_eq!(unpack(r"qwe\\5r\").unwrap(), r"qwe\\\\\r\");
    }
}
