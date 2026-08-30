+++
title = "How many lines of code are in Android 17?"
description = "I just reran a tool over Android 17 code - Here are the results!"
date = 2026-06-23
template = "blog-entry.html"
+++

Android 17 has roughly *113 million* lines of code running on the average device. If you include test cases and developer tooling you get 309 million lines in the AOSP 17, that's roughly 60 million more than I found in Android 16. Refer to my [earlier blog post](/blog/size-aosp14) on this matter for contextualization. Interestingly the absolute number of Rust lines has increased, probably because of Googles efforts to replace core dependencies with memory safe alternatives, while the share dropped to what looks like more test cases for code in unsafe languages. The source code is of course [published](https://github.com/derdilla/aosp-analyzer). I used tokei 14.0.0 for the core line counting.

## Quick facts

- There are over <b>309 million</b> source code lines contributing to Android
- Roughly <b>113 million</b> lines of code run on the average device
- <b>105 million</b> lines of comments and documentation tell the developers what the code does
- <b>9%</b> of lines are empty

## Doughnut chart

{{ <include_html path="content/_html_blobs/sizeofaosp17chart.html" /> }}

{{ <include_html path="content/_html_blobs/sizeaosp17table.html" /> }}
