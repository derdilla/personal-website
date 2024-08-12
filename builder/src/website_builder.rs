use std::{fs, io};
use std::io::{Read, Write};
use std::path::PathBuf;
use crate::builder::{BuildProcedureBuildError};
use crate::fs_tree::ParsedFsEntry;
use crate::ir::{IR};

pub struct Website {
    pages: Vec<(PathBuf, Vec<u8>)>,
}

impl Website {
    pub fn build(source: &IR) -> Result<Self, BuildProcedureBuildError> {
        let mut build_pages = Vec::new();

        println!("Copying static assets:");
        build_pages.append(&mut source.static_assets.clone());
        build_pages.push((PathBuf::from("layout.css"), source.layout_css.as_bytes().to_vec()));
        build_pages.push((PathBuf::from("style.css"),  source.style_css.as_bytes().to_vec()));

        println!("Building pages:");
        let build_scripts = source.pages.filter("yml");
        let total = build_scripts.len();
        let mut page_count = 0;
        for (mut path, build_script) in build_scripts {
            if let ParsedFsEntry::BuildProcedure(build_script) = build_script {
                path.set_extension("html");
                let path = PathBuf::from(path.strip_prefix("pages/").unwrap_or(&path));
                page_count += 1;
                println!("> {} ({} / {})", &path.to_str().unwrap(), &page_count, &total);
                let html = build_script.execute(&source)?;

                build_pages.push((path, html.as_bytes().to_vec()));
            }
        }

        Ok(Website { pages: build_pages })
    }

    pub fn write(&self, out: &PathBuf) -> bool {
        if out.is_file() {
            panic!("Can't write to file")
        } else if !out.exists() {
            if fs::create_dir_all(&out).is_err() {
                panic!("Can't create out dir")
            }
        } else if out.is_dir() {  } else { panic!("????") };

        if out.read_dir().unwrap().count() > 0 {
            panic!("Out dir is not empty")
        }

        for (sub_path, content) in &self.pages {
            Self::write_to_pathbuf(&out.join(sub_path), content).unwrap();
        }
        true
    }

    fn write_to_pathbuf(path: &PathBuf, content: &Vec<u8>) -> std::io::Result<()> { // TODO: evaluate and remove?
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = fs::File::create(path)?;
        file.write_all(content)?;
        Ok(())
    }
}
