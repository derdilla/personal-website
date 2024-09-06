use std::fs;
use std::io::Write;
use std::path::PathBuf;
use itertools::Itertools;
use lewp_css::domain::at_rules::font_face::FontDisplay::fallback;
use regex::{Regex, Replacer};

use crate::builder::BuildProcedureBuildError;
use crate::fs_tree::ParsedFsEntry;
use crate::ir::IR;
use crate::sitemapper::SiteMapBuilder;

pub struct Website {
    pub pages: Vec<(PathBuf, Vec<u8>)>,
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
        let mut sitemap = SiteMapBuilder::new((&source.config.url).clone());
        for (mut path, build_script) in build_scripts {
            if let ParsedFsEntry::BuildProcedure(build_script) = build_script {
                path.set_extension("html");
                let path = PathBuf::from(path.strip_prefix("pages/").unwrap_or(&path));
                page_count += 1;
                println!("> {} ({} / {})", &path.to_str().unwrap(), &page_count, &total);
                let html = build_script.execute(&source)?;

                let html = html.as_bytes().to_vec();
                sitemap.add(path.to_str().unwrap().to_string(), &html);
                build_pages.push((path, html));
            }
        }

        println!("Creating aliases:");
        let mut aliases = Vec::new();
        for (path, content) in &build_pages {
            let path = path.to_str().unwrap();
            if path.ends_with(".html") && !path.ends_with("index.html") {
                let name = path.split('/').last().unwrap().strip_suffix(".html").unwrap();
                println!("> {}", &path);

                let cannonical = format!("<link rel=\"canonical\" href=\"/{path}\" />");
                let content = String::from_utf8(content.clone()).expect("html files are utf8")
                    .replacen("</head>", format!("{cannonical}</head>").as_str(), 1)
                    .as_bytes().to_vec();
                
                let idx_path = path.replace(format!("{name}.html").as_str(), format!("{name}/index.html").as_str());
                println!("  - {}", &idx_path);
                aliases.push((PathBuf::from(idx_path), content));
            }
        }
        println!("> {} aliases created", &aliases.len());

        build_pages.append(&mut aliases);

        println!("Building sitemap:");
        build_pages.push((PathBuf::from("sitemap.xml"), sitemap.build().as_bytes().to_vec()));

        Ok(Website { pages: build_pages })
    }

    /// Validates generated files for unresolved variables and components.
    ///
    /// When errors are found they get printed and false gets returned.
    pub fn validate(&self) -> bool {
        println!("validating...");
        const RED: &'static str = "\x1b[31m";
        const CLEAR: &'static str = "\x1b[0m";

        let mut valid = true;
        let regex = Regex::new(r"\{\{ [^\s]* }}").unwrap();
        for (path, content) in self.pages.clone() {
            if path.extension().is_some_and(|e| e.to_str().unwrap() == "html") {
                if regex.is_match(&String::from_utf8(content).unwrap()) {
                    eprintln!("{RED}ERROR{CLEAR}: Unresolved variable in generated file {}", path.display());
                    valid = false;
                }
            }
        }
        valid
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

        println!("writing {:?}", &path);
        let mut file = fs::File::create(path)?;
        file.write_all(content)?;
        Ok(())
    }
}

mod test {
    use regex::Regex;

    #[test]
    fn validation_regex_works() {
        let regex = Regex::new(r"\{\{ [^\s]* }}").unwrap();
        assert!(regex.is_match("{{ test }}"));
        assert!(regex.is_match("{{ test/csom }}"));
        assert!(!regex.is_match("{{ test }"));
    }
}