use std::collections::HashMap;
use std::{fs, io};
use std::path::PathBuf;

use crate::fs_tree::FsTree;

/// Source files loaded into memory.
///
/// Not responsible for validating the contents of said files.
#[derive(Debug)]
pub struct SourceDir {
    /// Contents of the website.yml file.
    pub website_yml: String,

    /// Template file names and content.
    ///
    /// Example entry: `"base-page.html", "<!DOCTYPE html>..."`
    pub templates: HashMap<String, String>,

    /// Component ids and file content.
    ///
    /// Example entry: `"footer", "<footer>...</footer>"`
    pub components: HashMap<String, String>,

    pub layout_css: String,

    pub style_css: String,

    pub pages: FsTree,

    pub static_files: Vec<(PathBuf, Vec<u8>)>
}

impl SourceDir {

    pub fn load(root: PathBuf) -> Result<Self, SourceDirOpenError> {
        let website_yml = Self::read_website(&root)?;
        let templates = Self::read_templates(&root)?;
        let components = Self::read_components(&root)?;
        let layout = Self::read_layout(&root)?;
        let style = Self::read_style(&root)?;
        let pages = FsTree::load(&root.join("pages"));
        let mut static_files = Vec::new();
        if let Err(_) =  Self::collect_files(root.join("static"), &root.join("static"), &mut static_files) {
            return Err(SourceDirOpenError::NoSuchDirectory(String::from("static")));
        }

        Ok(SourceDir {
            website_yml,
            templates,
            components,
            layout_css: layout,
            style_css: style,
            pages,
            static_files,
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

    /// Recursively read all files relative to a root dir
    fn collect_files(dir: PathBuf, prefix_dir: &PathBuf, files: &mut Vec<(PathBuf, Vec<u8>)>) -> io::Result<()> {
        for entry in dir.read_dir()? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                Self::collect_files(path, &prefix_dir, files)?;
            } else if path.is_file() {
                let mut file_content = fs::read(&path)?;
                let path = path.canonicalize().unwrap();
                let pre = prefix_dir.canonicalize().unwrap();
                let relative_path = path.strip_prefix(pre).expect("couldn't follow paths").to_path_buf();

                files.push((relative_path, file_content));
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum SourceDirOpenError {
    MissingFile(String),
    NoSuchDirectory(String),

}