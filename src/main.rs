mod sequence;

use std::env;
use std::fs;
use std::io::{self, Read};
use std::process::ExitCode;

struct Options {
    lenient: bool,
    path: Option<String>,
}

fn parse_args() -> Result<Options, String> {
    let mut lenient = false;
    let mut path = None;

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--lenient" => lenient = true,
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
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

    Ok(Options { lenient, path })
}

fn print_usage() {
    eprintln!("usage: emojiseq [--lenient] [FILE]");
    eprintln!();
    eprintln!("Reads one emoji sequence per line, given as whitespace-separated");
    eprintln!("hex codepoints (an optional 'U+' prefix is allowed). Anything after");
    eprintln!("a ';' or '#' on a line is treated as a comment. Reads stdin if FILE");
    eprintln!("is omitted.");
}

/// Parses one line into codepoints. Returns None for lines that are blank
/// or comment-only once the trailing comment is stripped.
fn parse_line(line: &str) -> Result<Option<Vec<u32>>, String> {
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
        match parse_line(line) {
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

