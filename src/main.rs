use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    let working_dir = if args.len() >= 3 {
        let path = PathBuf::from(&args[2]);
        if !path.is_dir() {
            eprintln!("Directory doesn't exist: {}", &args[2]);
            print_usage();
            return;
        }
        path
    } else {
        env::current_dir().expect("Can't access current working directory")
    };


    let command = &args[1];
    match command.as_str() {
        "assemble" => assemble(),
        "pack" => pack(),
        "compile" => compile(),
        _ => {
            eprintln!("Unknown command: {command}");
            print_usage();
        },
    }

}

fn print_usage() {
    eprintln!("Usage: builder <command> [path]");
}
