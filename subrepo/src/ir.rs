use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Debug;

use lewp_css::Stylesheet;
use scraper::Html;
use yaml_rust2::{Yaml, YamlLoader};

use crate::fs_tree::{ParsedFsTree, ParsedFsTreeParseError};
use crate::source_dir::{FsTree, SourceDir};

/// Intermediate representation of the [SourceDir].
pub struct IR {
    /// Config from website.yml.
    pub config: Yaml,

    pub templates: HashMap<String, FwHTML>,

    pub components: HashMap<String, FwHTML>,

    pub layout_css: Stylesheet,

    pub style_css: Stylesheet,

    pub pages: ParsedFsTree,
}

impl IR {
    pub fn new(data: SourceDir) -> Result<Self, SourceFormatError> {
        let config = Self::load_config(data.website_yml.as_str())?;
        let templates = Self::load_templates(data.templates)?;
        let components = Self::load_components(data.components)?;
        let layout_css = Self::load_css(data.layout_css.as_str(), "layout.css")?;
        let style_css = Self::load_css(data.style_css.as_str(), "style.css")?;
        let pages = Self::load_pages(data.pages)?;
        // TODO: validate pages tree

        Ok(IR{
            config,
            templates,
            components,
            layout_css,
            style_css,
            pages,
        })
    }

    fn load_config(data: &str) -> Result<Yaml, SourceFormatError> {
        match YamlLoader::load_from_str(data) {
            Ok(yaml) => {
                if let Some(yaml) = yaml.first() {
                    Ok(yaml.clone())
                } else {
                    Err(SourceFormatError::BadWebsiteYaml)
                }
            }
            Err(_) => Err(SourceFormatError::InvalidYaml(String::from("website.yml")))
        }
    }

    fn load_templates(data: HashMap<String, String>) -> Result<HashMap<String, FwHTML>, SourceFormatError> {
        let mut loaded = HashMap::new();
        for (k, v) in data {
            match FwHTML::template(v) {
                Ok(v) => { loaded.insert(k, v); },
                Err(e) => return Err(SourceFormatError::InvalidTemplateHTML(k, e)),
            }
        };
        Ok(loaded)
    }

    fn load_components(data: HashMap<String, String>) -> Result<HashMap<String, FwHTML>, SourceFormatError> {
        let mut loaded = HashMap::new();
        for (k, v) in data {
            match FwHTML::new(v) {
                Ok(v) => { loaded.insert(k, v); },
                Err(e) => return Err(SourceFormatError::InvalidTemplateHTML(k, e)),
            }
        };
        Ok(loaded)
    }

    fn load_css(data: &str, filename: &str) -> Result<Stylesheet, SourceFormatError> {
        match Stylesheet::parse(data) {
            Err(err) => {
                return Err(SourceFormatError::InvalidCSS(CssParseError {
                    filename: String::from(filename),
                    source_location: err.location,
                    error_class: format!("{:?}", err.kind),
                }));
            }
            Ok(css) => Ok(css)
        }
    }

    fn load_pages(data: FsTree) -> Result<ParsedFsTree, SourceFormatError> {
        match data.parse() {
            Ok(tree) => Ok(tree),
            Err(err) => Err(SourceFormatError::InvalidFsTree(err)),
        }
    }
}

#[derive(Debug)]
pub enum SourceFormatError {
    InvalidYaml(String),
    InvalidTemplateHTML(String, FwHTMLError),
    /// website data is not in expected format.
    BadWebsiteYaml,
    InvalidCSS(CssParseError),
    InvalidFsTree(ParsedFsTreeParseError),
}

#[derive(Debug)]
pub struct CssParseError {
    filename: String,
    source_location: lewp_css::cssparser::SourceLocation,
    /// A [CustomParseError] debug text.
    error_class: String,
}

#[derive(Debug)]
pub struct FwHTML {
    /// Raw html as present in the template
    data: String,

    used_variables: Vec<String>,
    used_components: Vec<String>,
    used_class_names: Vec<String>,
}

impl FwHTML {
    fn template(data: String) -> Result<Self, FwHTMLError> {
        if !data.starts_with("<!DOCTYPE html>") {
            return Err(FwHTMLError::NotATemplate);
        }
        Self::new(data)
    }
    fn new(data: String) -> Result<Self, FwHTMLError> {
        let document = Html::parse_document(&data);
        if !document.errors.is_empty() {
            // TODO: validate html
            // return Err(FwHTMLError::DocumentContainsErrors(document.errors))
        }
        let mut classes = analyzer::classes(document);

        let variables = analyzer::used_variables(data.as_str());
        let components = analyzer::used_components(data.as_str());

        Ok(FwHTML {
            data,
            used_variables: variables,
            used_components: components,
            used_class_names: classes,
        })
    }
}

#[derive(Debug)]
pub enum FwHTMLError {
    DocumentContainsErrors(Vec<Cow<'static, str>>),
    /// Doesn't start with "<!DOCTYPE html>"
    NotATemplate,
}

mod analyzer {
    use ego_tree::iter::Edge;
    use once_cell::sync::Lazy;
    use regex::Regex;

    pub fn classes(html: scraper::Html) -> Vec<String> {
        let mut classes = Vec::new();
        for node in html.tree.root().traverse() {
            if let Edge::Open(node) = node {
                if let Some(node) = node.value().as_element(){
                    for c in node.classes() {
                        classes.push(c.to_string());
                    }
                }
            }
        }
        classes
    }

    static VAR_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{\{ (\w*) }}")
        .expect("Failed to compile classes regex"));
    pub fn used_variables(html: &str) -> Vec<String> {
        extract_regex_captures(&VAR_REGEX, html)
    }

    static COMP_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{\{ components/(\w*) }}")
        .expect("Failed to compile classes regex"));
    pub fn used_components(html: &str) -> Vec<String> {
        extract_regex_captures(&COMP_REGEX, html)
    }

    fn extract_regex_captures(regex: &Lazy<Regex, fn() -> Regex>, haystack: &str) -> Vec<String> {
        regex.captures_iter(haystack)
            .map(|e| e.get(1))
            .filter(|e| e.is_some())
            .map(|e| e.unwrap().as_str().to_string())
            .collect::<Vec<String>>()
    }
}