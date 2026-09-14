//! Omni Compiler Driver CLI Entry Point
//! Fully featured argument parser supporting input files, optimization levels, and output targets.

use std::env;
use std::fs;
use std::path::PathBuf;

pub struct Args {
    pub input_file: Option<PathBuf>,
    pub output_file: Option<PathBuf>,
    pub opt_level: u8,
    pub emit_native: bool,
}

pub fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut input_file = None;
    let mut output_file = None;
    let mut opt_level = 0;
    let mut emit_native = false;

    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-o" | "--output" => {
                if let Some(val) = iter.next() {
                    output_file = Some(PathBuf::from(val));
                } else {
                    return Err("Missing value for output flag".into());
                }
            }
            "-O" | "--opt-level" => {
                if let Some(val) = iter.next() {
                    opt_level = val.parse().unwrap_or(0);
                } else {
                    return Err("Missing value for optimization level".into());
                }
            }
            "--native" => {
                emit_native = true;
            }
            _ if !arg.starts_with('-') => {
                input_file = Some(PathBuf::from(arg));
            }
            _ => {
                return Err(format!("Unknown flag: {}", arg));
            }
        }
    }

    Ok(Args { input_file, output_file, opt_level, emit_native })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match parse_args(&args) {
        Ok(parsed) => {
            if let Some(input) = parsed.input_file {
                let source_code = match fs::read_to_string(&input) {
                    Ok(content) => content,
                    Err(e) => {
                        eprintln!("Error reading file {:?}: {}", input, e);
                        std::process::exit(1);
                    }
                };

                if parsed.emit_native {
                    match omni_codegen::compile_to_object(&source_code) {
                        Ok(bytes) => {
                            let out_path =
                                parsed.output_file.unwrap_or_else(|| PathBuf::from("output.o"));
                            if let Err(e) = fs::write(&out_path, bytes) {
                                eprintln!("Failed to write object file: {}", e);
                                std::process::exit(1);
                            }
                            println!("Successfully compiled native object to {:?}", out_path);
                        }
                        Err(e) => {
                            eprintln!("Codegen error: {}", e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    match omni_machine::bridge::execute_source(&source_code) {
                        Ok(result) => {
                            println!("Exit code: {}", result);
                        }
                        Err(e) => {
                            eprintln!("Execution error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            } else {
                println!("Omni Systems Programming Language Compiler v1.0.0.0");
                println!("Usage: omni-driver [OPTIONS] <INPUT_FILE>");
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
