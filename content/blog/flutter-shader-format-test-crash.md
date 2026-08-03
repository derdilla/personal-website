+++
title = "ink_sparkle.frag could not be decoded"
description = "Public documentation on simple way to fix weird exception in flutter test"
date = 2026-02-20
template = "blog-entry.html"
+++

After updating my flutter version to 3.41.2 and depencencies on linux I kept getting the following error when running `flutter test`:

```
══╡ EXCEPTION CAUGHT BY FLUTTER TEST FRAMEWORK ╞════════════════════════════════════════════════════
The following _Exception was thrown running a test:
Exception: Asset 'shaders/ink_sparkle.frag' manifest could not be decoded: INVALID_ARGUMENT:
Unsupported runtime stages format version. Expected 1, got 0.

When the exception was thrown, this was the stack:
#0      new FragmentProgram._fromAsset (dart:ui/painting.dart:5337:7)
#1      FragmentProgram.fromAsset.<anonymous closure> (dart:ui/painting.dart:5365:39)
#12     FakeAsync.run.<anonymous closure>.<anonymous closure> (package:fake_async/fake_async.dart:189:18)
#13     FakeAsync.flushMicrotasks (package:fake_async/fake_async.dart:200:32)
#14     AutomatedTestWidgetsFlutterBinding.runTest.<anonymous closure> (package:flutter_test/src/binding.dart:2206:17)
(elided 25 frames from dart:async and package:stack_trace)
```

According to folks on discord this is caused by shader format changes. The fix is simple:

```sh
flutter clean
```

I should have probably done that anyways after upgrading.

## Update 3 Aug. 26

Another problem with the same solution:

```
Launching lib/main.dart on Linux in debug mode...
/home/derdilla/code/blood-pressure-monitor-fl/app/linux/flutter/ephemeral/.plugin_symlinks/jni/src/dartjni.h:166:48: error: incompatible pointer types passing 'JNIEnv **' (aka 'const struct JNINativeInterface_ ***') to parameter of type 'void **' [-Wincompatible-pointer-types]
/home/derdilla/code/blood-pressure-monitor-fl/app/linux/flutter/ephemeral/.plugin_symlinks/jni/src/dartjni.c:189:56: error: incompatible pointer types passing 'JNIEnv **' (aka 'const struct JNINativeInterface_ ***') to parameter of type 'void **' [-Wincompatible-pointer-types]
/home/derdilla/code/blood-pressure-monitor-fl/app/linux/flutter/ephemeral/.plugin_symlinks/jni/src/third_party/../dartjni.h:166:48: error: incompatible pointer types passing 'JNIEnv **' (aka 'const struct JNINativeInterface_ ***') to parameter of type 'void **' [-Wincompatible-pointer-types]
Building Linux application...                                           
Error: Build process failed
```
