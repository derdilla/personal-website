+++
title = "Path -> Tree"
description = "Algorithm for turning a list of file paths into a tree."
date = 2026-06-22
template = "blog-entry.html"
+++

A relatively simple problem I re-solved a couple of times over the last few months is constructing a hierarchical directory/file tree from a list of UNIX style file paths `/foo/bar/baz.txt`. This should print a nice tree:

```rust
fn example() {
    let root = FsEntryMetadata::new(vec![
        "/foo/bar/baz.txt".to_string(),
        "/foo/bar/foo.txt".to_string(),
        "/foo/baz.txt".to_string(),
    ]);

    dbg!(root);
    // foo
    // |  bar
    // |  |  baz.txt
    // |  |  foo.txt
    // |  baz.txt
}
```

The types I used look like you would expect:

```rust
#[derive(Debug)]
struct FsEntryMetadata {
    whole_path: String,
    name: String,
    entry_type: EntryType,
}

#[derive(Debug)]
enum EntryType {
    Directory { children: Vec<FsEntryMetadata> },
    File, // could contain content
}
```

I trimmed the code down a bit, and used basic types everywhere. It should still be quick to adapt.


```rust
pub fn insert(root: &mut FsEntryMetadata,
              mut path: String,
              whole_path_prefix: String,) -> Result<(), &'static str> {
    while path.starts_with("/") {
        path.remove(0);
    }

    let EntryType::Directory { children } = &mut root.entry_type else {
        return Err("Root is not a directory");
    };

    if let Some((name, path)) = path.split_once("/") {
        // it's a directory
        let whole_path = format!("{}/{}", whole_path_prefix, &name);
        if children.iter().find(|e| &e.name == &name).is_none() {
            let root = FsEntryMetadata {
                whole_path: whole_path.clone(),
                name: name.to_string(),
                entry_type: EntryType::Directory {
                    children: Vec::new(),
                },
            };
            children.push(root);
        }
        let new_root = children.iter_mut()
            .find(|e| e.name == name).unwrap();
        insert(
            new_root,
            path.to_string(),
            whole_path,
        )
    } else {
        // it's the last file
        let whole_path = format!("{}/{}", whole_path_prefix, &path);
        // Depending on whether you are fine with duplicates or trust
        // your input data, you may want to add an exists check here.
        children.push(FsEntryMetadata {
            name: path,
            entry_type: EntryType::File,
            whole_path,
        });
        Ok(())
    }

}
```

If you want to be fancy you can hide the function calling

```rust
impl FsEntryMetadata {
    fn new(files: Vec<String>) -> FsEntryMetadata {
        let mut root = FsEntryMetadata {
            whole_path: "/".to_string(),
            name: String::new(),
            entry_type: EntryType::Directory { children: vec![] },
        };

        for f in files {
            insert(&mut root, f, String::new()).unwrap();
        }
        root
    }
}
```
