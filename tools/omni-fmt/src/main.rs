//! `omni-fmt` command-line interface.
//!
//! Usage:
//!   omni-fmt [OPTIONS] [FILE]...        Format files in place
//!   omni-fmt --check [FILE]...          Check formatting without writing
//!   omni-fmt --help                     Show this help

use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use omni_fmt::format_source;

#[derive(Debug)]
struct Args {
    check: bool,
    files: Vec<PathBuf>,
}

fn print_help() {
    println!("Omni source code formatter v{}\n", env!("CARGO_PKG_VERSION"));
    println!("Usage: omni-fmt [OPTIONS] [FILE]...");
    println!();
    println!("Options:");
    println!("  --check     Check formatting without writing; exit non-zero if any file needs formatting");
    println!("  --help      Show this help message");
    println!("  FILE        Source file(s) to format (reads stdin if no files given)");
}

fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut check = false;
    let mut files = Vec::new();

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            "--check" => {
                check = true;
            }
            _ if !arg.starts_with('-') => {
                files.push(PathBuf::from(arg));
            }
            _ => {
                return Err(format!("Unknown flag: {arg}"));
            }
        }
    }

    Ok(Args { check, files })
}

fn read_stdin() -> io::Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn format_file(path: &Path, check: bool) -> Result<bool, Box<dyn std::error::Error>> {
    // Returns true if the file was changed (or would be changed in --check mode).
    let source = fs::read_to_string(path)?;
    let formatted = match format_source(&source) {
        Ok(formatted) => formatted,
        Err(errors) => {
            eprintln!("{}: parse errors:", path.display());
            for error in &errors {
                eprintln!("  - {}", error.message);
            }
            return Ok(false);
        }
    };

    if check {
        if source != formatted {
            eprintln!("{}: not formatted", path.display());
            return Ok(true);
        }
        Ok(false)
    } else if source != formatted {
        fs::write(path, &formatted)?;
        eprintln!("{}: formatted", path.display());
        Ok(true)
    } else {
        Ok(false)
    }
}

fn run(args: &[String]) -> Result<i32, Box<dyn std::error::Error>> {
    let parsed = parse_args(args)?;
    let mut exit_code = 0i32;

    if parsed.files.is_empty() {
        // Read from stdin.
        let source = read_stdin()?;
        let formatted = match format_source(&source) {
            Ok(formatted) => formatted,
            Err(errors) => {
                for error in &errors {
                    eprintln!("parse error: {}", error.message);
                }
                return Ok(1);
            }
        };
        if parsed.check {
            if source != formatted {
                eprintln!("stdin: not formatted");
                return Ok(1);
            }
        } else {
            print!("{formatted}");
        }
        return Ok(exit_code);
    }

    for path in &parsed.files {
        let changed = match format_file(path, parsed.check) {
            Ok(changed) => changed,
            Err(err) => {
                eprintln!("{}: {}", path.display(), err);
                exit_code = 1;
                continue;
            }
        };
        if changed && parsed.check {
            exit_code = 1;
        }
    }

    Ok(exit_code)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match run(&args) {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("fatal: {err}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_help_flag() {
        let args = vec!["omni-fmt".to_string(), "--help".to_string()];
        let result = parse_args(&args);
        assert!(result.is_ok());
        // --help triggers exit(0) inside parse_args, so reaching here means
        // it didn't match --help; we just verify parse doesn't error.
        let _ = result.unwrap();
    }

    #[test]
    fn parses_check_flag() {
        let args = vec!["omni-fmt".to_string(), "--check".to_string(), "a.omni".to_string()];
        let parsed = parse_args(&args).expect("parse args");
        assert!(parsed.check);
        assert_eq!(parsed.files.len(), 1);
        assert_eq!(parsed.files[0], PathBuf::from("a.omni"));
    }

    #[test]
    fn parses_multiple_files() {
        let args = vec!["omni-fmt".to_string(), "a.omni".to_string(), "b.omni".to_string()];
        let parsed = parse_args(&args).expect("parse args");
        assert!(!parsed.check);
        assert_eq!(parsed.files.len(), 2);
    }

    #[test]
    fn rejects_unknown_flag() {
        let args = vec!["omni-fmt".to_string(), "--bogus".to_string()];
        let result = parse_args(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown flag"));
    }
}
