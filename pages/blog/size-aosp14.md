You probably came here because you searched for how much code on your device runs. The rough answer is 66 million for explanations why putting a number on this is misleading read the next section. If you want to explore explore the data yourself there are two more detailed sections below.

## To be Android or not to be Android

If you want to be extremely liberal in your definition, you can consider android an ecosystem. That would include everything from the bootloader to developer documentation to store apps to the smartwatch that only works on android. This would be practically impossible to count and easily reach the billions. An extremely conservative approach would look at the bare minimum lines of OS code that need to be compiled to pass the official [Compatibility Test Suite](https://source.android.com/docs/compatibility/cts). While theoratically feasible and potentially useful the average user probably wouldn't recognize this as a mobile operating system.

<aside><img src="/assets/android-levels.svg" alt="Diagram ilustrating the different levels of android" title="Levels Android" width="300" height="300"></aside>

As a middle ground this counting examines the contents of the main [AOSP repo](https://source.android.com). A current checkout of the repo is roughly 168 GiB large this is however on only code but also documnetation, developer tools and other ecosystem components. To break down this code I created [aosp-analyzer](https://github.com/NobodyForNothing/aosp-analyzer) a set of scripts and a program that produces a handy HTML file (a slightly modified version is used for the interactive statistics at the end of this article). Let's break it down some more:

It depends on androids `repo` tool to obtain the source code and [tokei](https://github.com/XAMPPRocky/tokei) to do the heavy line counting work. During line counting I do a preliminary categorization of line data based on the top level directories. After those first two slow steps it begins its actual analysis work: It seperates test code and documentation from the rest *([1](https://github.com/NobodyForNothing/aosp-analyzer/blob/main/visualizer/src/extractor.rs#L16-L66))*, removes data only files (mainly random test data) *([2](https://github.com/NobodyForNothing/aosp-analyzer/blob/main/visualizer/src/extractor.rs#L71-L73))* and assembles the html *([3](https://github.com/NobodyForNothing/aosp-analyzer/blob/main/visualizer/src/format.rs#L57-L62))*. In case you are curious about the details, I think the code is more explicit than this text could ever be.

{{ aosp-data }}

[android-levels]: /assets/android-levels.svg "Levels of android"