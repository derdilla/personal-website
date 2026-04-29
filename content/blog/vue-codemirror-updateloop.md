+++
title = "Reactivity doesn't mix well with yjs"
description = "Surprising interaction between vuejs yjs and codemirror. Or how I fixed a `EditorView.update are not allowed while an update is in progress`"
date = 2026-04-29
template = "blog-entry.html"
+++

When you have your `Y.Doc` in an object with deep reactivity like `reactive()`, `yCollab` from `y-codemirror.next` hits you with an error like this as soon as you start typing:

```
CodeMirror plugin crashed: Error: Calls to EditorView.update are not allowed while an update is in progress
   update index.js:7826
   dispatchTransactions index.js:7787
   dispatch index.js:7814
   _observer NextJS
   callAll function.js:20
   callEventHandlerListeners EventHandler.js:87
   callTypeObservers AbstractType.js:254
   _callObserver YText.js:923
   cleanupTransactions Transaction.js:283
   callAll function.js:20
   cleanupTransactions Transaction.js:320
   transact Transaction.js:443
   transact Doc.js:185
   update NextJS
   update index.js:1467
   updatePlugins index.js:7990
   update index.js:7888
   dispatchTransactions index.js:7787
   dispatch index.js:7814
   applyDOMChangeInner index.js:4271
   applyDOMChange index.js:4224
   flush index.js:7337
   observer index.js:7019
   DOMObserver index.js:7002
   _EditorView index.js:7796
   setupEditor TextEditor.vue:77
   setup TextEditor.vue:115
   callWithErrorHandling runtime-core.esm-bundler.js:199
   callWithAsyncErrorHandling runtime-core.esm-bundler.js:206
   call runtime-core.esm-bundler.js:880
   job reactivity.esm-bundler.js:1901
   callWithErrorHandling runtime-core.esm-bundler.js:199
   flushJobs runtime-core.esm-bundler.js:408
   promise callback*queueFlush runtime-core.esm-bundler.js:322
   queueJob runtime-core.esm-bundler.js:317
   scheduler runtime-core.esm-bundler.js:6208
   trigger reactivity.esm-bundler.js:267
   endBatch reactivity.esm-bundler.js:325
   trigger reactivity.esm-bundler.js:743
   set reactivity.esm-bundler.js:1061
   setSelected FileTree.vue:60
   callWithErrorHandling runtime-core.esm-bundler.js:199
   callWithAsyncErrorHandling runtime-core.esm-bundler.js:206
   invoker runtime-dom.esm-bundler.js:730
index.js:1356:17 
```

As far as I can tell, this is because vues deep reactivity wraps objects in a proxy which breaks the strict equality (`===`) used by some logic to detect foreign updates in the yjs editor binding. To fix exclude the Y.Doc from deep equality. For example by marking it as raw:

```ts
this.doc = markRaw(newDoc);
```
