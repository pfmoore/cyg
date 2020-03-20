extern crate serde;
//extern crate toml;
extern crate glob;

use glob::glob_with;
use glob::GlobError;
use glob::MatchOptions;
use std::env;
use std::iter::Iterator;
use std::path::PathBuf;
use std::result::Result;
use toml;
use serde::Deserialize;
use std::option::Option;
use std::fs;
use std::ffi::OsStr;

#[derive(Deserialize)]
struct Cyg {
    base: String,
    exe: Option<String>,
}

const GLOB_CHARS: [char; 3] = ['?', '*', '['];

fn is_literal(s: &str) -> bool {
    !s.chars().any(|c| GLOB_CHARS.contains(&c))
}

// Silently treat invalid patterns as literal.
// If we hit an error while matching, return it.
fn gen<I>(args: I) -> Result<Vec<PathBuf>, GlobError>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let mut paths : Vec<PathBuf> = Vec::new();
    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: true,
        require_literal_leading_dot: false,
    };

    for arg in args {
        // Glob needs a &str value.
        let arg = arg.as_ref();
        if is_literal(arg) {
            paths.push(arg.into());
        } else {
            match glob_with(arg, options) {
                Ok(mut path_iter) => {
                    // If we match no items, return the pattern...
                    match path_iter.next() {
                        None => paths.push(arg.into()),
                        Some(p) => {
                            paths.push(p?);
                            for p in path_iter {
                                paths.push(p?);
                            }
                        }
                    }
                }
                Err(_) => {
                    paths.push(arg.into());
                }
            }
        }
    }
    Ok(paths)
}

fn main() {
    let contents = fs::read_to_string("Cygwin.toml")
        .expect("Something went wrong reading the file");
    let options: Cyg = toml::from_str(&contents).unwrap();
    let exe_op = PathBuf::from(options.exe.unwrap());
    let me = env::current_exe().unwrap();
    let exe_name = me.file_stem().unwrap();
    let mut args = env::args().skip(1);
    let command = if exe_name == "cyg" || exe_name == exe_op {
        OsStr::from(args.next().unwrap()).as_ref()
    } else {
        exe_name
    };
    // Can't do this all in one line - message about temporary value getting dropped...
    let me = me.file_name().unwrap();
    println!("me = {:?}, base = {}, exe = {:?}", me, options.base, options.exe);
    let mut path = PathBuf::from(options.base);
    path.push("bin");
    path.push(exe_name);
    path.set_extension(".exe");
    if path.exists() {
        println!("Found {:?}!", path);
    } else {
        println!("Not Found {:?}!", path);
    }
    // let package_info: Value = toml::from_str(toml_content)?;
    match gen(args) {
        Ok(p) => println!("{:?}", p),
        Err(_) => eprintln!("Error!"),
    }
}
