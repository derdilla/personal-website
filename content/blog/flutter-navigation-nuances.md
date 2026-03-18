+++
title = "Correct close confirmation in Flutter is hard"
description = "Small rant about implementing a seemingly simple feature in flutter."
date = 2026-03-18
template = "blog-entry.html"
+++

Flutters Navigation is _nuanced_. I had an interesting experience trying to implement a confirmation before poping a route today. Normally I would just use the forms standard [`canPop` property](https://api.flutter.dev/flutter/widgets/Form/canPop.html), but since I didn't use a standard form here this wasn't possible.

My first thought was to go to the dialogue's close button and add a wrapper like this:

```dart
if (await shouldPop() && context.mounted) {
  Navigator.pop(context, null);
}
```

When testing this I remembered that I had to worry about system navigation (like Android's back button). I wanted to use a `WillPopScope`, but that's deprecated and stopped working by now, so I had to roll with [`PopScope`](https://api.flutter.dev/flutter/widgets/PopScope-class.html). There you need to prohibit all pops and handle the attempts by propagating accepted pops (seems hacky lol). However, for some reason `canPop: false` won't guarantee `didPop` is never true, so watch out for it. That still didn't seem to handle my app bar navigation, so I ended up checking in **two** places:

```dart
PopScope(
  canPop: false,
  // Popping though system buttons
  onPopInvokedWithResult: (didPop, result) async {
    if (didPop) return;
    if (await shouldPop() && context.mounted) {
      Navigator.pop(context, result);
    }
  },
  child: FullscreenDialog(
    // Popping though in-app buttons
    canClose: shouldPop,
    body: AddEntryForm(...),
  ),
);
```

The full context is in [a PR](https://github.com/derdilla/blood-pressure-monitor-fl/pull/642).
