use std::{fs, io};
use std::path::PathBuf;

use either::Either;

use crate::builder::{BuildProcedure, BuildProcedureLoadError};

#[derive(Debug)]
pub struct FsTree {
    entry_name: String,
    /// If this is a file: this is the content if this is a dir these are the
    /// children.
    child: Either<String, Vec<FsTree>>,
    created: Option<u64>
}

impl FsTree {
    pub fn load(path: &PathBuf) -> Result<Self, FsTreeLoadError> {
        if path.is_file() {
            let content = match fs::read_to_string(&path) {
                Err(err) => return Err(FsTreeLoadError::CantReadPath(path.clone(), err)),
                Ok(content) => content,
            };
            if let Some(Some(file_name)) = path.file_name().map(|f| f.to_str()) {
                let created = Self::get_added_to_git_date(path.canonicalize().unwrap_or_else(|_| path.clone()));
                Ok(FsTree {
                    entry_name: file_name.to_string(),
                    child: Either::Left(content),
                    created,
                })
            } else {
                Err(FsTreeLoadError::UnparsableFilename(path.clone()))
            }

        } else if path.is_dir() {
            let read_dir = match fs::read_dir(&path) {
                Err(err) => return Err(FsTreeLoadError::CantReadPath(path.clone(), err)),
                Ok(c) => c
            };
            let mut children = Vec::new();
            for e in read_dir {
                match e {
                    Err(err) => return Err(FsTreeLoadError::CantReadPath(path.clone(), err)),
                    Ok(e) => {
                        let subtree = FsTree::load(&e.path())?;
                        children.push(subtree);
                    }
                }
            }
            if let Some(Some(file_name)) = path.file_name().map(|f| f.to_str()) {
                Ok(FsTree {
                    entry_name: file_name.to_string(),
                    child: Either::Right(children),
                    created: None,
                })
            } else {
                Err(FsTreeLoadError::UnparsableFilename(path.clone()))
            }
        } else {
            Err(FsTreeLoadError::IsSymlink(path.clone()))
        }
    }

    fn get_added_to_git_date(path: PathBuf) -> Option<u64> {
        let out = std::process::Command::new("git")
            .arg("log")
            .arg("--pretty=format:%at") // https://git-scm.com/docs/pretty-formats#Documentation/pretty-formats.txt-ematem
            .arg(path)
            .output().ok()?;
        if !out.status.success() {
            return None;
        }
        let out = String::from_utf8(out.stdout).ok()?;
        let out = out.split("\n").last()?;
        out.parse().ok()
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
                    created: self.created,
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
                    created: self.created,
                })
            }
        }
    }

    // TODO: add pub util getters.
}

#[derive(Debug)]
pub enum FsTreeLoadError {
    CantReadPath(PathBuf, io::Error),
    UnparsableFilename(PathBuf),
    IsSymlink(PathBuf),
}

#[derive(Debug, Clone)]
pub struct ParsedFsTree {
    pub name: String,
    pub content: ParsedFsEntry,
    pub created: Option<u64>
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
