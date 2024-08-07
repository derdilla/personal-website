use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

use either::Either;
use yaml_rust2::{Yaml, YamlLoader};

#[derive(Debug)]
pub struct FsTree {
    entry_name: String,
    /// If this is a file: this is the content if this is a dir these are the
    /// children.
    child: Either<String, Vec<FsTree>>,
}

impl FsTree {
    pub fn load(path: &PathBuf) -> Self {
        // TODO: don't panic on error
        if path.is_file() {
            let content = fs::read_to_string(&path)
                .expect(format!("Couldn't read {}", &path.to_str().unwrap()).as_str());
            FsTree {
                entry_name: path.file_name().unwrap().to_str().unwrap().to_string(),
                child: Either::Left(content),
            }
        } else if path.is_dir() {
            let read_dir = fs::read_dir(&path)
                .expect(format!("Couldn't read {}", &path.to_str().unwrap()).as_str());
            let mut children = Vec::new();
            for e in read_dir {
                let e = e.expect(format!("Couldn't read {}", &path.to_str().unwrap()).as_str());
                children.push(FsTree::load(&e.path()));
            }
            FsTree {
                entry_name: path.file_name().unwrap().to_str().unwrap().to_string(),
                child: Either::Right(children),
            }
        } else {
            panic!("Can't handle symlink at {}", &path.to_str().unwrap())
        }
    }

    pub fn parse(self) -> Result<ParsedFsTree, ParsedFsTreeParseError> {
        match self.child {
            Either::Left(content) => {
                let content = match PathBuf::from(&self.entry_name).extension() {
                    None => Ok(ParsedFsEntry::TextFile(content)),
                    Some(OsStr::new("yml")) => {
                        match YamlLoader::load_from_str(content.as_str()) {
                            Ok(yaml) => Ok(ParsedFsEntry::YamlFile(yaml)),
                            Err(err) => Err(ParsedFsTreeParseError::InvalidYaml(err)),
                        }
                    },
                    Some(OsStr::new("md")) => Ok(ParsedFsEntry::Markdown(content)),
                    Some(_) => Ok(ParsedFsEntry::TextFile(content)),
                }?;
                Ok(ParsedFsTree {
                    name: self.entry_name,
                    content,
                })
            }
            Either::Right(children) => {
                let mut parsed = Vec::new();
                for child in children {
                    let child = child.parse()?;
                    parsed.push(child);
                }
                Ok(ParsedFsTree {
                    name: self.entry_name,
                    content: ParsedFsEntry::Directory(parsed),
                })
            }
        }
    }

    // TODO: add pub util getters.
}

#[derive(Debug, Clone)]
pub struct ParsedFsTree {
    name: String,
    content: ParsedFsEntry,
}

#[derive(Debug, Clone)]
pub enum ParsedFsEntry {
    Directory(Vec<ParsedFsTree>),
    TextFile(String),
    YamlFile(Vec<Yaml>),
    /// Contents of a markdown file. No validation required
    Markdown(String),
}

#[derive(Debug, Clone)]
pub enum ParsedFsTreeParseError {
    InvalidYaml(yaml_rust2::ScanError)
}