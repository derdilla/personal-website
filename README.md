# Homepage

## Directory structure

### `builder`

Source code of a program that can be run in the root directory to build a static
website in a new the `out` directory.

#### Usage:

- `builder assemble`: merge custom files into a standard compliant website
- `builder pack`: `assemble` and include css directly in html files, tree-shake unused classes and attributes
- `builder compile`: `pack` and minify js and css to reduce size

### `static`

Static HTML documents, CSS styles and assets that will be copied in that 
structure to the *assemble*d website.


### `components`

HTML snippets that may contain variables and other components of the format 
`{{ <id> }}`, where id must be provided during the *assemble*-stage.

### `templates`

HTML pages containing markup of the format `{{ <id> }}` where id can be the name
of a component or a custom variable. In the *assemble*-stage first component 
names get resolved, followed by variable names. Where variable names come from
is specified in the `builder` code.

### `pages`

This is the main directory responsible for building the site. It contains yml 
files and other data required to build the site. Each yaml-file represents one 
generated html file at the yaml-files relative path pages dir with a similar 
name.

Each yaml file contains a `template:` key specifying the file under the 
"templates" directory. Example: `template: base-page.html`

It can also contain build steps under the `steps:` key. Each step defines 
variable names and their replacement. Variable names from the last name are 
still present if they weren't overridden.

*Variable* values can be simple Strings or a map that must contain a `type` key 
and a `value` or a `path` key. A value is another *variable* value (can be 
nested). Paths are relative to the pages dir and are read as strings. 

```yml
steps:
  - "Setup blog template":
      title: Hacking window movement
      content: "{{ components/blog-entry }}"
  - "Fill data":
      description: A short tale on the joy of microprojects.
      timestamp:
        type: unixTimestamp
        value: 1706394048
      content:
        type: Md
        path: blog/kinetic-windows.blog
```

At the end of steps there must be no unresolved variables.

#### Available types

| *type*        | Description                                                                                                                                                                                                                                                                                          |
|---------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| text          | Raw text to directly insert.                                                                                                                                                                                                                                                                         |
| unixTimestamp | Unix timestamp in seconds. Creates a `<time>` HTML element.                                                                                                                                                                                                                                          |
| Md            | Text in markdown format.                                                                                                                                                                                                                                                                             |
| index         | Requires a directory in the `path` argument. Performs the specified `steps` in every .yml file (except index.yml) in the specified directory on any *component* specified in a custom `itemTemplate` key. Additionally provides a `link` variable that links to the article generated for that item. |

### Files

- `page-modifications`: Automatically generated: required in CWD to build the sitemap.

## CSS

In general CSS is considered as introducing too much complexity and should be 
reduced as far as possible. When using CSS is appropriate it is split between 
the `layout.css` and `style.css` files with distinct responsibility. 

The complexity of CSS used for layout should be reduced as far as possible. 
CSS used for styling must not shift element position on load, but may be used to
provide colors, fonts and responsiveness. 

Required CSS classes are automatically resolved and no manual imports are 
required. Compilation will fail if names collide or couldn't be resolved.  

## Assembly

TODO: rework section
Posts in `posts/*.blog` insert a blog-post to `{{ latest }}` in the blog 
component and a blog-entry component in the base-page template under the `/blog/` 
URL path.

- `components/blog-post.html` contains the "title", "description" date "date" variables read from `posts/*.blog`.
- `components/blog-entry.html` contains the "title", "description", formated "timestamp" and "content" variables read from `posts/*.blog`.

The `home.html` and `blog.html` components get inserted in a base-page template
and *assemble*d to the `/index.html` and `/blog/index.html` paths respectively


# TODO:

### Before release:

- sort blog posts in index
- validate for unresolved vars

### After:

- https://developers.google.com/search/docs/appearance/structured-data
- rss feed