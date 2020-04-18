extern crate glob;
extern crate toml;

use std::env;
use std::process::Command;
use std::ffi::OsStr;
use std::path::PathBuf;

use glob::glob_with;
use glob::MatchOptions;

use toml::Value;

const GLOB_CHARS: [char; 3] = ['?', '*', '['];

fn is_literal(s: &str) -> bool {
    !s.chars().any(|c| GLOB_CHARS.contains(&c))
}

fn run<E, S, I>(exe: E, args: I) -> ()
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
    E: AsRef<OsStr>,
    E: std::fmt::Debug,
{
    let result = Command::new(&exe)
        .args(args)
        .env("CYGWIN", "noglob")
        .status();

    match result {
        Ok(status) =>
            std::process::exit(status.code().unwrap_or(1)),
        Err(e) => {
            // If exe is a path, we can use exe.display()...
            // I know it *is* a path, just need to get the types right.
            eprintln!("Could not execute command {:?}: {}", exe, e);
            std::process::exit(1);
        }
    }
}

fn expand(args: &mut std::env::ArgsOs) -> Vec::<std::ffi::OsString> {
    let mut result = Vec::<std::ffi::OsString>::new();
    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: true,
        require_literal_leading_dot: false,
    };

    for arg in args {
        if let Some(arg_str) = arg.to_str() {
            if is_literal(arg_str) {
                result.push(arg);
            } else {
                match glob_with(arg_str, options) {
                    Err(_) => result.push(arg),
                    Ok(mut path_iter) => {
                        match path_iter.next() {
                            None => result.push(arg),
                            Some(p) => {
                                result.push(p.unwrap().into());
                                for p in path_iter {
                                    result.push(p.unwrap().into());
                                }
                            }
                        }
                    }
                }
            }
        } else {
            result.push(arg);
        }
    }
    result
}

fn command(args: &mut std::env::ArgsOs) -> std::ffi::OsString {
    let me = env::current_exe() /* Result<PathBuf> */
        .unwrap();
    let cmd = match me.file_stem() {
        None => args.next().unwrap(),
        Some(name) => if name == "cyg" { args.next().unwrap() } else { name.to_os_string() }
    };
    cmd
}

fn cygwin_base() -> PathBuf {
    // Config file is next to the executable
    let mut config = env::current_exe() /* Result<PathBuf> */
        .unwrap();
    config.set_file_name("Cygwin.toml");
    let contents = std::fs::read_to_string(config)
        .expect("Something went wrong reading the file");
    let val = contents.parse::<Value>().unwrap();
    // TODO: Default to the "Cygwin64" directory alongside this exe
    PathBuf::from(val["base"].as_str().unwrap_or("E:\\Utils\\Cygwin64"))
}

fn main() {
    let mut args = env::args_os();
    args.next(); // Skip the program name
    let mut exe = cygwin_base();
    exe.push("bin");
    exe.push(command(&mut args));
    exe.set_extension("exe");
    let args = expand(&mut args);
    run(exe, args);
}
