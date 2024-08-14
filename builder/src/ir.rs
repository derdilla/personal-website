use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::PathBuf;

use lewp_css::Stylesheet;
use scraper::Html;
use serde::Deserialize;

use crate::builder;
use crate::builder::ValueGenerationError;
use crate::fs_tree::{FsTree, ParsedFsTree, ParsedFsTreeParseError};
use crate::source_dir::SourceDir;

/// Intermediate representation of the [SourceDir].
pub struct IR {
    pub config: WebsiteConf,

    pub templates: HashMap<String, FwHTML>,

    pub components: HashMap<String, FwHTML>,

    pub layout_css: String,

    pub style_css: String,

    pub pages: ParsedFsTree,

    pub static_assets: Vec<(PathBuf, Vec<u8>)>,
}

impl IR {
    pub fn new(data: SourceDir) -> Result<Self, SourceFormatError> {
        let config = Self::load_config(data.website_yml.as_str())?;
        let templates = Self::load_templates(data.templates)?;
        let components = Self::load_components(data.components)?;
        let pages = Self::load_pages(data.pages)?;
        // TODO: validate pages tree and css

        Ok(IR{
            config,
            templates,
            components,
            layout_css: data.layout_css,
            style_css: data.style_css,
            pages,
            static_assets: data.static_files,
        })
    }

    fn load_config(data: &str) -> Result<WebsiteConf, SourceFormatError> {
        serde_yml::from_str(data)
            .map_err(|err| SourceFormatError::InvalidYaml(String::from("website.yml"), err))
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
        // FIXME: wait until css parser supports ":has" attribute
        match Stylesheet::parse(/*data*/"") {
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

#[derive(Debug, Deserialize)]
pub struct WebsiteConf {
    pub url: String,
}

#[derive(Debug)]
pub enum SourceFormatError {
    /// Name of the file that has invalid yaml
    InvalidYaml(String, serde_yml::modules::error::Error),
    /// File name, err
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

#[derive(Debug, Clone)]
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

    /// Inserts components and variables as long as possible.
    pub fn resolved<F>(&self, components: &HashMap<String, FwHTML>, variables: &HashMap<&String, F>) -> Result<Self, FwHTMLResolveError>
    where
        F: Fn() -> Result<String, builder::ValueGenerationError>,
    {
        // TODO: proper error propagation
        let mut html = self.data.clone();

        for comp_name in &self.used_components {
            let comp = match components.get(comp_name) {
                None => return Err(FwHTMLResolveError::MissingComponent(comp_name.clone())),
                Some(comp) => comp,
            };
            html = html.replace(format!("{{{{ components/{comp_name} }}}}").as_str(), comp.data.as_str());
        }

        for var_name in &self.used_variables {
            if let Some(var) = variables.get(var_name) {
                let var = match var() {
                    Err(err) => return Err(FwHTMLResolveError::VariableError(var_name.clone(), Box::new(err))),
                    Ok(var) => var,
                };
                html = html.replace(format!("{{{{ {var_name} }}}}").as_str(), var.as_str());
            }
        }

        let mut new = match Self::new(html) {
            Err(err) => return Err(FwHTMLResolveError::GeneratesErrorInDocument(err)),
            Ok(n) => n,
        };
        while new.used_variables.iter().any(|e| variables.contains_key(e))
            || new.used_components.iter().any(|e| components.contains_key(e)) {
            let tmp = variables.clone();
            //println!("{:?}", &new.used_variables.iter().filter(|e| tmp.contains_key(e.clone())));
            //println!("{:?}", new.data);
            new = new.resolved(&components, &variables)?;
            new = match Self::new(new.data) {
                Err(err) => return Err(FwHTMLResolveError::GeneratesErrorInDocument(err)),
                Ok(n) => n,
            };
        }

        Ok(new)
    }

    pub fn output(&self) -> String {
        // TODO: validate
        self.data.clone()
    }
}

#[derive(Debug)]
pub enum FwHTMLError {
    DocumentContainsErrors(Vec<Cow<'static, str>>),
    /// Doesn't start with "<!DOCTYPE html>"
    NotATemplate,
}

#[derive(Debug)]
pub enum FwHTMLResolveError {
    MissingComponent(String),
    VariableError(String, Box<ValueGenerationError>),
    GeneratesErrorInDocument(FwHTMLError),
}

mod analyzer {
    use ego_tree::iter::Edge;
    use itertools::Itertools;
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

    static VAR_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{\{ ([\w\-_]*) }}")
        .expect("Failed to compile classes regex"));
    pub fn used_variables(html: &str) -> Vec<String> {
        extract_regex_captures(&VAR_REGEX, html)
    }

    static COMP_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{\{ components/([\w\-_]*) }}")
        .expect("Failed to compile classes regex"));
    pub fn used_components(html: &str) -> Vec<String> {
        extract_regex_captures(&COMP_REGEX, html)
    }

    fn extract_regex_captures(regex: &Lazy<Regex, fn() -> Regex>, haystack: &str) -> Vec<String> {
        regex.captures_iter(haystack)
            .map(|e| e.get(1))
            .filter(|e| e.is_some())
            .map(|e| e.unwrap().as_str().to_string())
            .sorted().dedup()
            .collect::<Vec<String>>()
    }
}
