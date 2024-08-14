use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc};
use itertools::Itertools;

pub struct SiteMapBuilder {
    root: String,
    entries: Vec<SiteMapEntry>,
    resolver: ModificationTimestampResolver,
}

impl SiteMapBuilder {
    pub fn new(mut root: String) -> Self {
        if !root.starts_with("http") {
            root = format!("https://{root}")
        }
        if !root.ends_with("/") {
            root = format!("{root}/")
        }
        SiteMapBuilder {
            root,
            entries: Vec::new(),
            resolver: ModificationTimestampResolver::new(),
        }
    }

    /// Add the page at a [path] relative to the URL (e.g. index.html) to the
    /// sitemap.
    ///
    /// The last modified timestamp is automatically determined from the content hash
    pub fn add(&mut self, path: String, content: &Vec<u8>) {
        let path = path.strip_prefix("/").unwrap_or(path.as_str());
        let path = path.strip_suffix("index.html").unwrap_or(path);
        let path = format!("{}{path}", self.root);
        let modified = self.resolver.get_change_time(content);
        self.entries.push(SiteMapEntry{
            loc: path,
            lastmod: modified.format("%Y-%m-%d").to_string(),
            priority: None,
        })
    }

    pub fn build(&self) -> String {
        let entries = self.entries.iter()
            .map(|e| e.build())
            .join("");
        format!("<urlset>{entries}</urlset>")
    }
}
struct SiteMapEntry {
    loc: String,
    lastmod: String,
    priority: Option<String>,
}

impl SiteMapEntry {
    fn build(&self) -> String {
        if let Some(priority) = &self.priority {
            format!("<url><loc>{}</loc><lastmod>{}</lastmod><priority>{priority}</priority></url>", self.loc, self.lastmod)
        } else {
            format!("<url><loc>{}</loc><lastmod>{}</lastmod></url>", self.loc, self.lastmod)
        }
    }
}


struct ModificationTimestampResolver {
    old_txt: String,
    queried: Vec<(String, i64)>,
}

impl ModificationTimestampResolver {
    pub fn new() -> Self {
        let txt = fs::read_to_string("page-modifications");
        if txt.is_err() {
            const YELLOW: &'static str = "\x1b[33m";
            const CLEAR: &'static str = "\x1b[0m";
            eprintln!("{YELLOW}WARNING{CLEAR}: No existing page modifications file detected. Creating a new one...")
        }
        ModificationTimestampResolver {
            old_txt: txt.unwrap_or(String::new()),
            queried: Vec::new(),
        }
    }

    pub fn get_change_time(&mut self, content: &Vec<u8>) -> DateTime<Utc> {
        let mut hash = DefaultHasher::new();
        content.hash(&mut hash);
        let hash = hash.finish().to_string();
        let hash = hash.as_str();

        // format: "<hash>,<unixTime>
        let timestamp = self.old_txt
            .split("\n")
            .filter_map(|hash_date| hash_date
                .strip_prefix(hash)
                .and_then(|e| e.strip_prefix(','))
                .and_then(|e| e.parse::<i64>().ok())
            )
            .next();

        let timestamp = timestamp.unwrap_or(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64);
        self.queried.push((hash.to_string(), timestamp.clone()));

        DateTime::from_timestamp_millis(timestamp).unwrap()
    }
}

impl Drop for ModificationTimestampResolver {
    fn drop(&mut self) {
        let text = self.queried.iter()
            .map(|(hash, time)| format!("{hash},{time}"))
            .join("\n");

        if fs::write("page-modifications", text.as_bytes()).is_err() {
            eprintln!("Unable to save page modification dates to disk:\n\n{text}");
        }
    }
}