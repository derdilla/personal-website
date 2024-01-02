# Adding blog posts
To add a blog post put a file with the .blog extension in the posts folder, everything before the dot will be the url (`somepost.blog` -> `../blog/somepost`). When building a html file will be generated and the post will be at the top of the `{{content}}` section defined in `index.html`. Blog posts use the same template.

The format is not intended for potentially mallicious blog posts.

## file format
The files are saved with a standard unix line ending (`\n`).

Required:
- First line title for the blog itself and the overview
- Second line short description for the overview
- Third line unix timestamp in miliseconds

After an empty line begins the post content. It can contain HTML tags, but no css-classes are guaranteed. Everything gets put in paragraphs, empty lines (`\n\n`) get converted to `</p><p>`.
