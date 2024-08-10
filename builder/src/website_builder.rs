use std::fs;
use std::io::Write;
use std::path::PathBuf;
use crate::builder::{BuildProcedureBuildError};
use crate::fs_tree::ParsedFsEntry;
use crate::ir::{IR};

pub struct Website {
    pages: Vec<(PathBuf, String)>,
}

impl Website {
    pub fn build(source: &IR) -> Result<Self, BuildProcedureBuildError> {
        let mut build_pages = Vec::new();
        let build_scripts = source.pages.filter("yml");
        let total = build_scripts.len();
        for (mut path, build_script) in build_scripts {
            if let ParsedFsEntry::BuildProcedure(build_script) = build_script {
                path.set_extension("html");
                println!("> {}", &path.to_str().unwrap());
                println!(" building...");
                let html = build_script.execute(&source)?;

                build_pages.push((path, html));
                println!(" build {} / {} pages", &build_pages.len(), &total);
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
            println!("sub_path {}", content);
            Self::write_string_to_pathbuf(&out.join(sub_path), content).unwrap();
        }
        true
    }

    fn write_string_to_pathbuf(path: &PathBuf, content: &str) -> std::io::Result<()> { // TODO: evaluate and remove?
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = fs::File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}

/*struct BuildScript {
    template: String,
    steps: Vec<BuildStep>,
}

impl BuildScript {
    pub fn execute(&self, data: &IR) -> Result<String, BuildScriptBuildError> {
        println!("  - loading template:");
        let mut template: FwHTML = match data.templates.get(&self.template) {
            None => return Err(BuildScriptBuildError::TemplateNotFound(self.template.clone())),
            Some(template) => template.resolved(&data.components, &HashMap::new()),
        };
        println!("  - executing steps:");
        for step in &self.steps {
            println!("    >");
            template = template.resolved(&data.components, &step.variables);
        }

        Ok(template.output())
    }
}

#[derive(Debug)]
pub enum BuildScriptFormatError {
    Empty,
    IsAList,
    NoTemplateSpecified,
}

#[derive(Debug)]
pub enum BuildScriptBuildError {
    TemplateNotFound(String)
}

struct BuildStep {
    name: String,
    variables: HashMap<String, Yaml>
}

impl BuildStep {
    fn new(data: &yaml::Hash) -> Option<BuildStep> {
        // TODO: decide on step name
        /*let mut key = data.();
        if key.len() != 1 {
            panic!("Expect only one key because of context. {:?}", data);
        }
        let key = key.next().unwrap();
        let value = data.get(&key)?;

        let name = key.as_str().unwrap().to_string();*/
        let variables = data;
        let variables = variables.iter()
            .map(|(k ,v)| {
                let k = k.clone().into_string();
                if k.is_none() {
                    None
                } else {
                    // TODO: custom variable types as specified in readme
                    Some((k.unwrap(), v.clone()))
                }
            })
            .filter(|e| e.is_some())
            .map(|e| e.unwrap())
            .collect::<HashMap<String, Yaml>>();

        Some(BuildStep {
            name: String::from("TODO get step name"),
            variables,
        })
    }
}

enum VariableData {
    /// Contains raw text.
    Text(String),
    UnixTimestamp(Instant),
    /// Contains Markdown text.
    MD(String),
    /// Contains path and template.
    Index(String, String)
}

impl VariableData {

}*/