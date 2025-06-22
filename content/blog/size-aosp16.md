+++
title = "How many lines of code are in Android 16?"
description = "Android 16 brings 250 million lines of code and comments to the table. Let's look at percentages!"
date = 2025-06-12
template = "blog-entry.html"
+++

[Android 16](https://www.android.com/intl/en_us/new-features-on-android/?category=android-16) just released, and with it the code size grew again (+50 mil. LoC since Android 14). But you probably came here because you searched for how much code runs on your device. The rough answer is **92 million** for explanations why putting a number on this is misleading read [my last post on that matter](/blog/size-aosp14). If you want to explore explore the data yourself there are two more details below. As always the raw data and source code is [here](https://github.com/derdilla/aosp-analyzer).

## Quick facts

- There are over <b>250 million</b> source code lines contributing to Android
- Roughly <b>92 million</b> lines of code run on the average device
- <b>83 million</b> lines of comments and documentation tell the developers what the code does
- <b>8.8%</b> of lines are empty

## Doughnut chart

This time I included this nice interactive doughnut chart of the programming languages used (unfortunately requires JavaScript).

{{ sizeofaosp16chart() }}

{{ sizeaosp16table() }}
