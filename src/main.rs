use std::env;
use std::path::PathBuf;
use crate::ir::IR;
use crate::source_dir::{SourceDir, SourceDirOpenError};

mod source_dir;
mod page_builder;
mod ir;
mod fs_tree;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    let command = &args[1];
    let command = match command.as_str() {
        "assemble" => Command::Assemble,
        "pack" => Command::Pack,
        "compile" => Command::Compile,
        _ => {
            eprintln!("Unknown command: {command}");
            print_usage();
            return;
        },
    };

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

    println!("Reading data from '{}'...", &working_dir.to_str().unwrap());

    let source = SourceDir::load(working_dir);
    if let Err(err) = &source {
        match err {
            SourceDirOpenError::MissingFile(f) => eprintln!("Couldn't read file: {f}"),
            SourceDirOpenError::NoSuchDirectory(d) => eprintln!("No such directory: {d}"),
        }
        return;
    }
    let source = source.unwrap();
    let source = IR::new(source).unwrap();

    // TODO:
    // Fail on:
    // - Variables without value
    // - Missing css classes or tags
    // Warn on:
    // - Unused files
    // - Missing index.html
}

enum Command {
    Assemble,
    Pack,
    Compile,
}

fn print_usage() {
    eprintln!("Usage: builder <command> [path]");
}
