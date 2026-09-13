mod sequence;

use std::env;
use std::fs;
use std::io::{self, Read};
use std::process::ExitCode;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Format {
    Hex,
    Text,
}

struct Options {
    lenient: bool,
    format: Format,
    path: Option<String>,
}

fn parse_args() -> Result<Options, String> {
    let mut lenient = false;
    let mut format = Format::Hex;
    let mut path = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--lenient" => lenient = true,
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            "--format" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--format requires a value (hex or text)".to_string())?;
                format = parse_format(&value)?;
            }
            other if other.starts_with("--format=") => {
                format = parse_format(&other["--format=".len()..])?;
            }
            other if other.starts_with('-') => {
                return Err(format!("unrecognized option: {}", other));
            }
            other => {
                if path.is_some() {
                    return Err("only one input file may be given".to_string());
                }
                path = Some(other.to_string());
            }
        }
    }

    Ok(Options {
        lenient,
        format,
        path,
    })
}

fn parse_format(value: &str) -> Result<Format, String> {
    match value {
        "hex" => Ok(Format::Hex),
        "text" => Ok(Format::Text),
        other => Err(format!(
            "unrecognized --format value '{}' (expected 'hex' or 'text')",
            other
        )),
    }
}

fn print_usage() {
    eprintln!("usage: emojiseq [--lenient] [--format hex|text] [FILE]");
    eprintln!();
    eprintln!("In the default 'hex' format, reads one emoji sequence per line, given");
    eprintln!("as whitespace-separated hex codepoints (an optional 'U+' prefix is");
    eprintln!("allowed). Anything after a ';' or '#' on a line is treated as a");
    eprintln!("comment.");
    eprintln!();
    eprintln!("In 'text' format, reads one sequence per line as raw UTF-8 emoji text");
    eprintln!("(each Unicode scalar value on the line becomes one codepoint of the");
    eprintln!("sequence). Anything after a ';' is treated as a comment; '#' is not a");
    eprintln!("comment marker in this mode since it's also a valid keycap base.");
    eprintln!();
    eprintln!("Reads stdin if FILE is omitted.");
}

/// Parses one hex-format line into codepoints. Returns None for lines that
/// are blank or comment-only once the trailing comment is stripped.
fn parse_line_hex(line: &str) -> Result<Option<Vec<u32>>, String> {
    let cut = line
        .find([';', '#'])
        .map(|idx| &line[..idx])
        .unwrap_or(line);
    let cut = cut.trim();
    if cut.is_empty() {
        return Ok(None);
    }

    let mut cps = Vec::new();
    for token in cut.split_whitespace() {
        let digits = token.strip_prefix("U+").unwrap_or(token);
        let cp = u32::from_str_radix(digits, 16)
            .map_err(|_| format!("'{}' is not a hex codepoint", token))?;
        cps.push(cp);
    }
    Ok(Some(cps))
}

/// Parses one text-format line into codepoints: each Unicode scalar value
/// on the line (after stripping a trailing ';' comment and surrounding
/// whitespace) becomes one codepoint of the sequence. Unlike hex format,
/// '#' is not a comment marker here since it's a legitimate keycap base.
fn parse_line_text(line: &str) -> Option<Vec<u32>> {
    let cut = line.find(';').map(|idx| &line[..idx]).unwrap_or(line);
    let cut = cut.trim();
    if cut.is_empty() {
        return None;
    }
    Some(cut.chars().map(|c| c as u32).collect())
}

fn format_sequence(cps: &[u32]) -> String {
    cps.iter()
        .map(|cp| format!("{:04X}", cp))
        .collect::<Vec<_>>()
        .join(" ")
}

fn run(opts: &Options) -> io::Result<bool> {
    let text = match &opts.path {
        Some(path) => fs::read_to_string(path)?,
        None => {
            let mut buf = String::new();
            io::stdin().lock().read_to_string(&mut buf)?;
            buf
        }
    };

    let mut ok_count = 0;
    let mut bad_count = 0;

    for (line_no, line) in text.lines().enumerate() {
        let line_no = line_no + 1;
        let parsed = match opts.format {
            Format::Hex => parse_line_hex(line),
            Format::Text => Ok(parse_line_text(line)),
        };
        match parsed {
            Ok(None) => continue,
            Ok(Some(cps)) => match sequence::validate(&cps, opts.lenient) {
                Ok(()) => {
                    ok_count += 1;
                    println!("ok   {}", format_sequence(&cps));
                }
                Err(reason) => {
                    bad_count += 1;
                    println!("bad  {}: {}", format_sequence(&cps), reason);
                }
            },
            Err(reason) => {
                bad_count += 1;
                println!("bad  line {}: {}", line_no, reason);
            }
        }
    }

    println!("{} ok, {} bad", ok_count, bad_count);
    Ok(bad_count == 0)
}

// Also usable as a Rust lib reader via stdin for quick manual checks:
//   echo "1F469 200D 1F4BB" | emojiseq
fn main() -> ExitCode {
    let opts = match parse_args() {
        Ok(opts) => opts,
        Err(reason) => {
            eprintln!("emojiseq: {}", reason);
            print_usage();
            return ExitCode::from(2);
        }
    };

    match run(&opts) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(err) => {
            eprintln!("emojiseq: {}", err);
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_parses_plain_codepoints() {
        assert_eq!(parse_line_hex("1F469 200D 1F4BB").unwrap(), Some(vec![0x1F469, 0x200D, 0x1F4BB]));
    }

    #[test]
    fn hex_strips_u_plus_prefix() {
        assert_eq!(parse_line_hex("U+1F44B").unwrap(), Some(vec![0x1F44B]));
    }

    #[test]
    fn hex_strips_trailing_comment() {
        assert_eq!(parse_line_hex("1F44B ; waving hand").unwrap(), Some(vec![0x1F44B]));
        assert_eq!(parse_line_hex("1F44B # waving hand").unwrap(), Some(vec![0x1F44B]));
    }

    #[test]
    fn hex_blank_or_comment_only_line_is_none() {
        assert_eq!(parse_line_hex("").unwrap(), None);
        assert_eq!(parse_line_hex("   ").unwrap(), None);
        assert_eq!(parse_line_hex("; just a comment").unwrap(), None);
    }

    #[test]
    fn hex_rejects_non_hex_token() {
        assert!(parse_line_hex("not-hex").is_err());
    }

    #[test]
    fn text_converts_each_scalar_value() {
        // waving hand + skin tone modifier, as literal characters.
        let line = "\u{1F44B}\u{1F3FB}";
        assert_eq!(parse_line_text(line), Some(vec![0x1F44B, 0x1F3FB]));
    }

    #[test]
    fn text_keeps_hash_as_data_not_comment() {
        // keycap "#" sequence: '#', VS16, combining enclosing keycap.
        let line = "#\u{FE0F}\u{20E3}";
        assert_eq!(parse_line_text(line), Some(vec![0x23, 0xFE0F, 0x20E3]));
    }

    #[test]
    fn text_strips_trailing_semicolon_comment() {
        let line = "\u{1F44B} ; waving hand";
        assert_eq!(parse_line_text(line), Some(vec![0x1F44B]));
    }

    #[test]
    fn text_blank_or_comment_only_line_is_none() {
        assert_eq!(parse_line_text(""), None);
        assert_eq!(parse_line_text("   "), None);
        assert_eq!(parse_line_text("; just a comment"), None);
    }

    #[test]
    fn format_flag_parses_hex_and_text() {
        assert!(matches!(parse_format("hex"), Ok(Format::Hex)));
        assert!(matches!(parse_format("text"), Ok(Format::Text)));
        assert!(parse_format("json").is_err());
    }
}

