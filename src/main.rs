mod config;
mod engine;

use arboard::Clipboard;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();
    let exe_path = env::current_exe().unwrap_or_else(|_| PathBuf::from("arg-shim"));
    let exe_name = exe_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("arg-shim");
    
    // Check if we are running in "Management Mode" (invoked as arg-shim)
    let is_management_mode = exe_name.to_lowercase() == "arg-shim" 
                          || exe_name.to_lowercase() == "arg-shim.exe";
    
    let mut dry_run = false;
    
    // Default app identity is the executable name
    // If running as arg-shim, this can be overridden by --app
    let mut app_identity = exe_name.to_string();
    
    // We will collect arguments that should be processed by the shim here
    let mut raw_args_parts: Vec<String> = Vec::new();

    if is_management_mode {
        let mut i = 1;
        while i < args.len() {
            let arg = args[i].as_str();
            match arg {
                "--init" => {
                    let _ = config::create_default_config();
                    return;
                }
                "--help" | "-h" => {
                    print_help();
                    return;
                }
                "--version" | "-v" => {
                    println!("arg-shim {}", env!("CARGO_PKG_VERSION"));
                    return;
                }
                "--check" => {
                    dry_run = true;
                    i += 1;
                }
                "--app" => {
                    if i + 1 < args.len() {
                        app_identity = args[i + 1].clone();
                        i += 2;
                    } else {
                        println!("Error: --app requires a value");
                        return;
                    }
                }
                _ => {
                    // Stop parsing flags at the first non-flag or "--"
                    if arg == "--" {
                        i += 1;
                        // Add all remaining arguments
                        while i < args.len() {
                            raw_args_parts.push(args[i].clone());
                            i += 1;
                        }
                        break;
                    }
                    
                    // Treat as an argument
                    raw_args_parts.push(args[i].clone());
                    i += 1;
                }
            }
        }
    } else {
        // Not management mode: everything is raw args (except the executable path itself)
        if args.len() > 1 {
            raw_args_parts.extend_from_slice(&args[1..]);
        }
    }

    // Load Configuration
    // We pass the *real* exe_name to find the config file (e.g., putty.toml)
    let config = config::load(exe_name).unwrap_or_default();
    
    let raw_args_str = raw_args_parts.join(" ");

    let mut context = engine::Context {
        named: HashMap::new(),
        positional: &raw_args_parts,
        exe_name: &app_identity, // Use the identity (which might be overridden via --app)
        raw_args: raw_args_str.clone(),
    };

    // Process Rules
    let result = engine::process(&config.rules, &mut context);

    // Fallback logic
    let final_text = result.unwrap_or_else(|| {
        if config.fallback_raw {
            raw_args_str
        } else {
            String::new()
        }
    });

    if final_text.is_empty() {
        return;
    }

    if dry_run {
        println!("Dry run result:\n{}", final_text);
    } else {
        match Clipboard::new() {
            Ok(mut clipboard) => {
                if let Err(e) = clipboard.set_text(final_text) {
                    eprintln!("Error: Failed to set clipboard content: {}", e);
                }
            },
            Err(e) => eprintln!("Error: Failed to initialize clipboard: {}", e),
        }
    }
}

fn print_help() {
    println!(r###"arg-shim - A flexible CLI argument shim and transformer

USAGE:
    arg-shim [FLAGS] [ARGUMENTS...]

FLAGS:
    --init          Generate a default 'arg-shim.toml' configuration file in the current directory.
    --check <args>  Test transformation against <args> and print result without copying to clipboard.
    --app <name>    Override the application name for rule matching (useful with --check).
    -h, --help      Print this help message.
    -v, --version   Print version information.

TRANSFORMATION:
    When run without flags, arg-shim will intercept arguments and transform them 
    based on the loaded configuration files, then copy the result to the clipboard.

    Note: The flags above only work when the executable is named 'arg-shim'.
"###);
}