+++
title = "Fixing failing build_runner execution"
description = "Darts sqlite3 builds can fail, even if you didn't set them up"
date = 2026-03-22
template = "blog-entry.html"
+++

I recently got this error while trying to `dart run build_runner build` my app:

```
Running build hooks...Unhandled exception:
Bad state: Hash of downloaded file libsqlite3.x64.linux.so is e8f37ac037a255322e60d3b51f4280e328c3e548859b3f80db2524e27e0e88cf, expected 8323680aed1789f4dc5403feda6c3ebda7bd171a95bd7b0468a8e2ee415389ee.

  Building assets for package:sqlite3 failed.
  build.dart returned with exit code: 255.
  To reproduce run:
  (cd /home/derdilla/.pub-cache/hosted/pub.dev/sqlite3-3.2.0/; /home/derdilla/.local/share/flutter/bin/cache/dart-sdk/bin/dart --packages=/home/derdilla/code/blood-pressure-monitor-fl/app/.dart_tool/package_config.json /home/derdilla/code/blood-pressure-monitor-fl/app/.dart_tool/hooks_runner/sqlite3/6b4ebf10e7/hook.dill --config=/home/derdilla/code/blood-pressure-monitor-fl/app/.dart_tool/hooks_runner/sqlite3/6b4ebf10e7/input.json )
  stderr:
  Unhandled exception:
Bad state: Hash of downloaded file libsqlite3.x64.linux.so is e8f37ac037a255322e60d3b51f4280e328c3e548859b3f80db2524e27e0e88cf, expected 8323680aed1789f4dc5403feda6c3ebda7bd171a95bd7b0468a8e2ee415389ee.

  stdout:
  
          
Running build hooks...Error: Running build hooks failed.
```

This is somewhat surprising since I don't directly depend on sqlite3 and never set up anything build related myself. But sqflite does; and it expects a system package with a specific hash. This fails because my new CachyOS installation ships an architecture optimized binary, which of course has a different hash. To resolve this, take a look at the [sqlite3 documentation on hooks]( https://github.com/simolus3/sqlite3.dart/blob/main/sqlite3/doc/hook.md). For me that meant explicitly depending on their build by added the following hook to my `pubspec.yaml`:

```yaml
hooks:
  user_defines:
    sqlite3:
      source: sqlite3
```
