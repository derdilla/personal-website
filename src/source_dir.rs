use std::collections::HashMap;
use std::fmt::{Display, format, Formatter};
use std::fs;
use std::path::PathBuf;
use either::Either;

/// Source files loaded into memory.
///
/// Not responsible for validating the contents of said files.
#[derive(Debug)]
pub struct SourceDir {
    /// Contents of the website.yml file.
    website_yml: String,

    /// Template file names and content.
    ///
    /// Example entry: `"base-page.html", "<!DOCTYPE html>..."`
    templates: HashMap<String, String>,

    /// Component ids and file content.
    ///
    /// Example entry: `"footer", "<footer>...</footer>"`
    components: HashMap<String, String>,

    layout_css: String,

    style_css: String,

    pages: FsTree,
}

impl SourceDir {

    pub fn load(root: PathBuf) -> Result<Self, SourceDirOpenError> {
        let website_yml = Self::read_website(&root)?;
        let templates = Self::read_templates(&root)?;
        let components = Self::read_components(&root)?;
        let layout = Self::read_layout(&root)?;
        let style = Self::read_style(&root)?;
        let pages = FsTree::load(&root.join("pages")); // TODO

        Ok(SourceDir {
            website_yml,
            templates,
            components,
            layout_css: layout,
            style_css: style,
            pages,
        })
    }

    fn read_website(root: &PathBuf) -> Result<String, SourceDirOpenError> {
        match fs::read_to_string(root.join("website.yml")) {
            Err(_) => return Err(SourceDirOpenError::MissingFile(String::from("website.yml"))),
            Ok(txt) => Ok(txt),
        }
    }

    fn read_layout(root: &PathBuf) -> Result<String, SourceDirOpenError> {
        match fs::read_to_string(root.join("layout.css")) {
            Err(_) => return Err(SourceDirOpenError::MissingFile(String::from("layout.css"))),
            Ok(txt) => Ok(txt),
        }
    }

    fn read_style(root: &PathBuf) -> Result<String, SourceDirOpenError> {
        match fs::read_to_string(root.join("style.css")) {
            Err(_) => return Err(SourceDirOpenError::MissingFile(String::from("style.css"))),
            Ok(txt) => Ok(txt),
        }
    }

    fn read_templates(root: &PathBuf) -> Result<HashMap<String, String>, SourceDirOpenError> {
        let templates = root.join("templates");
        Self::read_dir(&templates)
    }

    fn read_components(root: &PathBuf) -> Result<HashMap<String, String>, SourceDirOpenError> {
        let components = root.join("components");
        let components = Self::read_dir(&components);
        components.and_then(|components| Ok(components.iter()
            .map(|(k, v)| {
                let k = k.strip_suffix(".html");
                (k, v)
            })
            .filter(|(k, _)| k.is_some())
            .map(|(k, v)| (k.unwrap().to_string(), v.to_string()))
            .collect::<HashMap<String, String>>())
        )
    }

    /// Attempt to load files at the top level of [dir] into memory.
    ///
    /// The resulting name is the file name as key and the content as value.
    fn read_dir(dir: &PathBuf) -> Result<HashMap<String, String>, SourceDirOpenError> {
        let files = match dir.read_dir() {
            Ok(d) => d,
            Err(_) => return Err(SourceDirOpenError::NoSuchDirectory(dir.to_str().unwrap().to_string())),
        };
        let mut loaded_files = HashMap::new();
        for t in files {
            if let Ok(e) = t {
                let path = &e.path();
                if !path.is_file() {
                    continue;
                }
                if let Ok(content) = fs::read_to_string(&path) {
                    let file_name = e.file_name().to_str().unwrap().to_string();
                    loaded_files.insert(file_name, content);
                } else {
                    return Err(SourceDirOpenError::MissingFile(path.to_str().unwrap().to_string()));
                }
            } else {
                return Err(SourceDirOpenError::NoSuchDirectory(dir.to_str().unwrap().to_string()));
            }
        };
        Ok(loaded_files)
    }
}

#[derive(Debug)]
pub enum SourceDirOpenError {
    MissingFile(String),
    NoSuchDirectory(String),

}

#[derive(Debug)]
pub struct FsTree {
    /// If this is a file: this is the content if this is a dir these are the
    /// children.
    child: Either<String, Vec<FsTree>>,
}

impl FsTree {
    fn load(path: &PathBuf) -> Self {
        // TODO: don't panic on error
        if path.is_file() {
            let content = fs::read_to_string(&path)
                .expect(format!("Couldn't read {}", &path.to_str().unwrap()).as_str());
            FsTree {
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
                child: Either::Right(children),
            }
        } else {
            panic!("Can't handle symlink at {}", &path.to_str().unwrap())
        }
    }

    // TODO: add pub util getters.
}