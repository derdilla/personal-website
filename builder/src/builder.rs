use std::cmp::PartialEq;
use std::collections::HashMap;
use itertools::Itertools;

use crate::fs_tree::ParsedFsEntry;
use crate::ir::{FwHTML, FwHTMLResolveError, IR};

#[derive(Debug, Clone)]
pub struct BuildProcedure {
    template: String,
    steps: Vec<Step>,
}

#[derive(Debug, Clone)]
pub struct Step {
    name: Option<String>,
    vars: HashMap<String, Value>,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    Text(String),
    Int(u64),
    UnixTimestamp {
        value: u64,
    },
    Md {
        path: String,
    },
    TextFile {
        path: String,
    },
    Index {
        path: String,
        /// Name of a *component*.
        item_template: String,
    },
}

impl BuildProcedure {
    pub fn new(s: &str) -> Result<BuildProcedure, BuildProcedureLoadError> {
        let deserialized: loader::BuildFile = match serde_yml::from_str(s) {
            Err(err) => return Err(BuildProcedureLoadError::FormatError(err)),
            Ok(v) => v,
        };

        let mut steps = Vec::new();
        for old_step in  deserialized.steps {
            let mut name = None;
            let mut vars: HashMap<String, Value> = HashMap::new();
            for (key, val) in old_step {
                if name.is_none() && val == loader::Value::None {
                    name = Some(key);
                    continue
                }
                let val = match val {
                    loader::Value::Tagged { inner } => match inner {
                        loader::ValueTyped::UnixTimestamp { value } => Value::UnixTimestamp{ value },
                        loader::ValueTyped::Md { path } => Value::Md{ path },
                        loader::ValueTyped::Text { path } => Value::TextFile{ path },
                        loader::ValueTyped::Index { path, item_template } => Value::Index{ path, item_template },
                    }
                    loader::Value::Text(txt) => Value::Text(txt),
                    loader::Value::Int(val) => Value::Int(val),
                    loader::Value::None => return Err(BuildProcedureLoadError::UnexpectNoneVariableValue),
                };
                vars.insert(key, val);
            }
            steps.push(Step { name, vars });
        }
        Ok(BuildProcedure {
            template: deserialized.template,
            steps,
        })
    }

    pub fn execute(&self, data: &IR,) -> Result<String, BuildProcedureBuildError> {
        let template: FwHTML = match data.templates.get(&self.template) {
            None => return Err(BuildProcedureBuildError::TemplateNotFound(self.template.clone())),
            Some(template) => template.clone(),
        };
        self.execute_with_template_override(data, template)
    }

    pub fn execute_with_template_override(&self, data: &IR, mut template: FwHTML) -> Result<String, BuildProcedureBuildError> {
        let mut vars = HashMap::new();
        for step in &self.steps {
            let mut step_vars = step.vars.iter()
                .map(|(k, v)| (k, || v.generate(&data)));
            vars.extend(&mut step_vars);

            template = match template.resolved(&data.components, &vars) {
                Err(err) => return Err(BuildProcedureBuildError::TemplateResolveError(err)),
                Ok(t) => t,
            };
        }

        Ok(template.output())
    }
}

#[derive(Debug)]
pub enum BuildProcedureLoadError {
    FormatError(serde_yml::modules::error::Error),
    UnexpectNoneVariableValue,
}

#[derive(Debug)]
pub enum BuildProcedureBuildError {
    /// Which template was not found.
    TemplateNotFound(String),
    /// Which build step couldn't resolve which variable.
    CantResolveVars(Option<String>, String),
    TemplateResolveError(FwHTMLResolveError),
}

impl Value {
    /// Turn the variable into a html compatible string.
    ///
    /// If the variable uses a path that isn't available resolving fails.
    pub fn generate(&self, data: &IR) -> Result<String, ValueGenerationError> {
        match self {
            Value::Text(txt) => Ok(txt.clone()),
            Value::Int(val) => Ok(val.to_string()),
            Value::UnixTimestamp { value } => {
                let timestamp = match chrono::DateTime::from_timestamp(*value as i64, 0) {
                    None => return Err(ValueGenerationError::UnixTimestampOutOfReach),
                    Some(time) => time,
                };

                let formal = timestamp.to_rfc3339();
                let pretty = timestamp.format("%Y-%m-%d").to_string();
                Ok(format!("<time datetime=\"{formal}\">{pretty}</time>").to_string())
            },
            Value::Md { path } => {
                if let Some(ParsedFsEntry::TextFile(md)) = data.pages.get(&format!("pages/{path}").to_string()) {
                    let parser = pulldown_cmark::Parser::new(&md);
                    let mut html = String::new();
                    pulldown_cmark::html::push_html(&mut html, parser);
                    Ok(html)
                } else {
                    Err(ValueGenerationError::FileDoesntExist(path.clone()))
                }
            }
            Value::TextFile { path } => {
                if let Some(ParsedFsEntry::TextFile(txt)) = data.pages.get(&format!("pages/{path}").to_string()) {
                    Ok(txt)
                } else {
                    Err(ValueGenerationError::FileDoesntExist(path.clone()))
                }
            }
            Value::Index { path, item_template } => {
                let dir = match data.pages.get(&format!("pages/{}", &path).to_string()) {
                    None => return Err(ValueGenerationError::NoDirAtIndexPath(path.clone())),
                    Some(dir) => dir,
                };
                if let ParsedFsEntry::Directory(children) = dir {
                    let mut html = String::new();
                    // Sorted in reverse order
                    let children = children.iter()
                        .sorted_by(|a, b| b.created.unwrap_or(0).cmp(&a.created.unwrap_or(0)));
                    for child in children {
                        let child = child.clone();
                        if let ParsedFsEntry::BuildProcedure(mut proc) = child.content {
                            if child.name == String::from("index.yml") {
                                continue;
                            }
                            if child.created.is_none() {
                                return Err(ValueGenerationError::IndexGitTimestampMissing(child.name))
                            }
                            let template = match data.components.get(item_template) {
                                None => return Err(ValueGenerationError::MissingComponent(item_template.clone())),
                                Some(t) => t,
                            };
                            let out_name = child.name.replace(".yml", ".html");
                            proc.steps.insert(0, Step {
                                name: Some(String::from("~~ index vars")),
                                vars: HashMap::from([(String::from("link"), Value::Text(out_name))])
                            });
                            let element_html = match proc.execute_with_template_override(data, template.clone()) {
                                Ok(html) => html,
                                Err(err) => return Err(ValueGenerationError::CantBuildIndexItem(child.name.clone(), err)),
                            };
                            // FIXME: component not found as template -> make template string
                            html += format!("\n{}", element_html).as_str();
                        }
                    }
                    Ok(html)
                } else {
                    Err(ValueGenerationError::NoDirAtIndexPath(path.clone()))
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum ValueGenerationError {
    FileDoesntExist(String),
    UnixTimestampOutOfReach,
    NoDirAtIndexPath(String),
    CantBuildIndexItem(String, BuildProcedureBuildError),
    MissingComponent(String),
    IndexGitTimestampMissing(String)
}

mod loader {
    use std::collections::HashMap;

    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub(super) struct BuildFile {
        pub template: String,
        pub steps: Vec<BuildStep>
    }

    pub(super) type BuildStep = HashMap<String, Value>;

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(untagged, rename_all = "camelCase")]
    pub(super) enum Value {
        Tagged {
            #[serde(flatten)]
            inner: ValueTyped,
        },
        Text(String),
        Int(u64),
        None,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(rename_all = "camelCase", tag = "type")] // , content = "value"
    pub(super) enum ValueTyped {
        UnixTimestamp {
            value: u64,
        },
        Md {
            // TODO: add value field and use options when needed.
            path: String,
        },
        Text {
            // TODO: add value field and use options when needed.
            path: String,
        },
        Index {
            path: String,
            #[serde(rename="itemTemplate")]
            item_template: String,
        },
    }


    mod tests {
        use crate::builder::loader::BuildFile;

        #[test]
        fn typed() {
            let deserialised: Result<BuildFile, _> = serde_yml::from_str(r#"
    template: base-page.html

    steps:
      - timestamp:
          type: unixTimestamp
          value: 1704204000
        text:
          type: md
          path: blog/size-aosp.blog
    "#);
            let deserialised = deserialised.unwrap();
            println!("{:?}", deserialised);
        }

        #[test]
        fn mixed() {
            let deserialised: Result<BuildFile, _> = serde_yml::from_str(r#"
    template: base-page.html

    steps:
      - "Setup blog template":
        title: "How many lines of code are in Android?"
        content: "{{ components/blog-entry }}"
      - "Fill data":
        description: "Here is how I found out that the AOSP has 2.5 million lines of code."
        timestamp:
          type: unixTimestamp
          value: 1704204000
        text:
          type: md
          path: blog/size-aosp.blog
    "#);
            let deserialised = deserialised.unwrap();
            println!("{:?}", deserialised);
        }
    }
}

mod tests {
    use std::collections::HashMap;
    use crate::builder::{BuildProcedure, Value};

    #[test]
    fn decodes_sample_blog_template() {
        let procedure = BuildProcedure::new(r#"template: base-page.html

steps:
  - "Setup blog template":
    title: How many lines of code are in Android?
    content: "{{ components/blog-entry }}"
  - "Fill data":
    description: Here is how I found out that the AOSP has 2.5 million lines of code.
    timestamp:
      type: unixTimestamp
      value: 1704204000
    text:
      type: md
      path: blog/size-aosp.blog

"#);
        let procedure = procedure.unwrap();
        assert_eq!(procedure.template, "base-page.html");
        let steps = procedure.steps;
        assert_eq!(steps.len(), 2);
        assert_eq!(steps.get(0).unwrap().name, Some(String::from("Setup blog template")));
        assert_eq!(steps.get(0).unwrap().vars, HashMap::from([
            (String::from("title"), Value::Text(String::from("How many lines of code are in Android?"))),
            (String::from("content"), Value::Text(String::from("{{ components/blog-entry }}"))),
        ]));
        assert_eq!(steps.get(1).unwrap().name, Some(String::from("Fill data")));
        assert_eq!(steps.get(1).unwrap().vars, HashMap::from([
            (String::from("description"), Value::Text(String::from("Here is how I found out that the AOSP has 2.5 million lines of code."))),
            (String::from("timestamp"), Value::UnixTimestamp{ value: 1704204000 }),
            (String::from("text"), Value::Md{ path: String::from("blog/size-aosp.blog") }),
        ]));

    }
}