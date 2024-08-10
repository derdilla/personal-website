use std::cmp::PartialEq;
use std::collections::HashMap;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use crate::fs_tree::ParsedFsEntry;
use crate::ir::{FwHTML, IR};

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
    Index {
        path: String,
        item_template: String,
    },
}

impl BuildProcedure {
    pub fn new(s: &str) -> Result<BuildProcedure, BuildProcedureLoadError> {
        let deserialized= serde_yml::from_str(s);
        let deserialized: loader::BuildFile = if let Err(err) = deserialized {
            return Err(BuildProcedureLoadError::FormatError(err))
        } else {
            deserialized.unwrap()
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

    pub fn execute(&self, data: &IR) -> Result<String, BuildProcedureBuildError> {
        let mut template: FwHTML = match data.templates.get(&self.template) {
            None => return Err(BuildProcedureBuildError::TemplateNotFound(self.template.clone())),
            Some(template) => template.resolved(&data.components, &HashMap::new()),
        };
        let mut vars = HashMap::new();
        for step in &self.steps {
            let step_vars = step.vars.iter()
                .map(|(k, v)| (k, v.generate(&data)));
            if step_vars.clone().any(|(k, v)| v.is_none()) {
                if let Some((var_name, _)) = step_vars.clone().filter(|(k, v)| v.is_none()).collect::<Vec<_>>().first() {
                    return Err(BuildProcedureBuildError::CantResolveVars(step.name.clone(), var_name.clone().clone()))
                }
            }

            let mut step_vars = step_vars
                .map(|(k, v)| (k, v.unwrap()));
            vars.extend(&mut step_vars);

            template = template.resolved(&data.components, &vars);
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
    CantResolveVars(Option<String>, String)
}

impl Value {
    /// Turn the variable into a html compatible string.
    ///
    /// If the variable uses a path that isn't available resolving fails.
    pub fn generate(&self, data: &IR) -> Option<String> {
        match self {
            Value::Text(txt) => Some(txt.clone()),
            Value::Int(val) => Some(val.to_string()),
            // TODO: make timestamp return one good html element
            Value::UnixTimestamp { value } => {
                let timestamp = chrono::DateTime::from_timestamp(*value as i64, 0).expect("out-of-range timestamp");

                let formal = timestamp.to_rfc3339();
                let pretty = timestamp.format("%Y-%m-%d").to_string(); // TODO:
                Some(format!("<time datetime=\"{formal}\">{pretty}</time>").to_string())
            },
            Value::Md { path } => {
                if let Some(ParsedFsEntry::TextFile(md)) = data.pages.get(&format!("pages/{path}").to_string()) {
                    let parser = pulldown_cmark::Parser::new(&md);
                    let mut html = String::new();
                    pulldown_cmark::html::push_html(&mut html, parser);
                    Some(html)
                } else {
                    eprintln!("{path}");
                    None
                }
            }
            Value::Index { .. } => {
                // TODO
                None
            }
        }
    }
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
        Index {
            path: String,
            item_template: String,
        },
    }


    mod tests {
        use super::BuildFile;

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