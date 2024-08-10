use std::fs;
use std::path::PathBuf;

use either::Either;
use yaml_rust2::{Yaml, YamlLoader};
use crate::builder::{BuildProcedure, BuildProcedureLoadError};

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
                let content = match PathBuf::from(&self.entry_name).extension().and_then(|e|e.to_str()) {
                    None => Ok(ParsedFsEntry::TextFile(content)),
                    Some("yml") => {
                        match BuildProcedure::new(content.as_str()) {
                            Ok(procedure) => Ok(ParsedFsEntry::BuildProcedure(procedure)),
                            Err(err) => Err(ParsedFsTreeParseError::InvalidBuildProcedure(PathBuf::from(&self.entry_name), err)),
                        }
                    },
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
    pub name: String,
    pub content: ParsedFsEntry,
}

#[derive(Debug, Clone)]
pub enum ParsedFsEntry {
    Directory(Vec<ParsedFsTree>),
    TextFile(String),
    BuildProcedure(BuildProcedure),
}

#[derive(Debug)]
pub enum ParsedFsTreeParseError {
    InvalidBuildProcedure(PathBuf, BuildProcedureLoadError)
}

impl ParsedFsTree {
    /// Gather all files with a matching [file_extension] and return their paths and content.
    ///
    /// Paths are relative to this [ParsedFsTree]s parent directory.
    pub fn filter(&self, file_extension: &str) -> Vec<(PathBuf, ParsedFsEntry)> {
        let path = PathBuf::from(&self.name);
        match &self.content {
            ParsedFsEntry::Directory(children) => children.iter()
                .flat_map(|c| c
                    .filter(file_extension)
                    .iter()
                    .map(|(child_path, entry)| (path.join(child_path), entry.clone()))
                    .collect::<Vec<_>>()
                )
                .collect::<Vec<(PathBuf, ParsedFsEntry)>>()
            ,
            content => {
                if path.extension().is_some_and(|e| e.to_str().is_some_and(|e| e == file_extension)) {
                    vec![(path, content.clone())]
                } else {
                    Vec::new()
                }
            },
        }
    }

    pub fn get(&self, path: &String) -> Option<ParsedFsEntry> {
        if &self.name == path { // only happens at the end
            Some(self.content.clone())
        } else if let ParsedFsEntry::Directory(children) = &self.content {
            let mut path_parts = path.splitn(2, '/');
            let dir_name = path_parts.next();
            let path= path_parts.next();
            if dir_name.is_some() && path.is_some() && self.name.as_str() == dir_name.unwrap() {
                let path = path.unwrap().to_string();
                for e in children {
                    let e = e.get(&path);
                    if e.is_some() {
                        return e;
                    }
                }
            }

            None
        } else {
            None
        }
    }
}
