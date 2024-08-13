use std::{env, fs};
use std::path::PathBuf;
use crate::fs_tree::{FsTreeLoadError, ParsedFsTreeParseError};

use crate::ir::{FwHTMLError, IR, SourceFormatError};
use crate::source_dir::{SourceDir, SourceLoadError};
use crate::website_builder::Website;

mod source_dir;
mod website_builder;
mod ir;
mod fs_tree;
mod builder;
mod sitemapper;

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
    // TODO: error building functions

    let source = match SourceDir::load(working_dir) {
        Err(err) => return eprintln!("{}", err_source_load(err)),
        Ok(source) => source,
    };
    let source = match IR::new(source) {
        Err(err) => return eprintln!("{}", err_source_format(err)),
        Ok(ir) => ir,
    };
    let website = Website::build(&source).unwrap();
    if PathBuf::from("out").exists() {
        fs::remove_dir_all("out").unwrap();
    }
    website.write(&PathBuf::from("out"));

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

fn err_source_load(err: SourceLoadError) -> String {
    match err {
        SourceLoadError::MissingFile(f) => format!("Couldn't read file: {f}"),
        SourceLoadError::NoSuchDirectory(d) => format!("No such directory: {d}"),
        SourceLoadError::BadFsTree(err) => format!("Can't load fs tree:\n\t{}", err_fs_tree_open(err))
    }
}

fn err_source_format(err: SourceFormatError) -> String {
    match err {
        SourceFormatError::InvalidYaml(file, yaml) => format!("Invalid yml file: {file}"),
        SourceFormatError::InvalidTemplateHTML(file, err) => format!("Invalid HTML in file: {file}\n\t{}", err_fw_html(err)),
        SourceFormatError::BadWebsiteYaml => "website.yml in not in expected format".to_string(),
        SourceFormatError::InvalidCSS(err) => panic!("TODO"),
        SourceFormatError::InvalidFsTree(err) => err_fs_tree_parse(err),
    }
}

fn err_fs_tree_open(err: FsTreeLoadError) -> String {
    match err {
        FsTreeLoadError::CantReadPath(path, err) => format!("{}: {err}", path.to_str().unwrap_or("NON_UTF8_PATH")),
        FsTreeLoadError::UnparsableFilename(path) => format!("Can't parse filename of: {}", path.to_str().unwrap_or("NON_UTF8_PATH")),
        FsTreeLoadError::IsSymlink(path) => format!("Unexpected symlink at: {}", path.to_str().unwrap_or("NON_UTF8_PATH")),
    }
}

fn err_fs_tree_parse(err: ParsedFsTreeParseError) -> String {
    match err {
        ParsedFsTreeParseError::InvalidBuildProcedure(path, err) => {
            todo!()
        }
    }
}

fn err_fw_html(err: FwHTMLError) -> String {
    todo!()
}
