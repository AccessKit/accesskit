# AccessKit Android adapter

This crate exposes [AccessKit](https://accesskit.dev/) trees to Android’s accessibility framework.

## Adding the dependency

```toml
[target.'cfg(target_os = "android")'.dependencies]
accesskit_android = { version = "0.4", features = ["embedded-dex"] }
```

Enable the optional `embedded-dex` feature to bundle the helper Java class (`dev.accesskit.android.Delegate`) directly into your native library. If you disable the feature you must package that class yourself (for example via Gradle).

## Two integration layers

- [`Adapter`](src/adapter.rs) is the low-level bridge. You embed it inside your own `View` subclass and call its methods from JNI when Android invokes `AccessibilityNodeProvider`.
- [`InjectingAdapter`](src/inject.rs) installs an `AccessibilityDelegate` and hover listener onto an existing `View`, posts callbacks to the UI thread, and forwards the framework requests to the low-level adapter. This is the simplest path when you can obtain a `View` handle (for example GameActivity’s surface).

## Using the low-level `Adapter`

1. **Store the adapter and handlers in your view.** Create `Adapter` plus `ActivationHandler` and `ActionHandler` instances when the host view is constructed.
2. **Forward framework queries.** Implement `createAccessibilityNodeInfo`, `findFocus`, `performAction`, and hover handling in Java/Kotlin, then call the corresponding Rust methods (`create_accessibility_node_info`, `find_focus`, `perform_action`, `on_hover_event`). Each method requires both the current `JNIEnv` and the host `View`.
3. **Deliver tree updates.** Call [`update_if_active`](src/adapter.rs#L224) from any thread with a closure that builds a `TreeUpdate`. When it returns [`QueuedEvents`](src/event.rs#L305), attach to the UI thread and call `raise(env, host)` to emit the queued `AccessibilityEvent`s. Never hold locks that your `View` might need while raising events; TalkBack can synchronously call back into your delegate.
4. **Handle action callbacks.** Your `ActionHandler` receives `ActionRequest` structs whenever TalkBack invokes a control. Apply the action to your UI, then call `update_if_active` again so the tree stays in sync.

## Using `InjectingAdapter`

```rust
let mut adapter = InjectingAdapter::new(&mut env, &host_view, activation_handler, action_handler);
adapter.update_if_active(|| build_tree_update());
```

- The helper registers an `AccessibilityDelegate` and `OnHoverListener`. If the view already has one, constructing `InjectingAdapter` will panic.
- All callbacks are posted to the UI thread via `View::post`, so you can create or drop the adapter from any thread.
- When the adapter is dropped it removes the delegate automatically.

## Testing

- Enable TalkBack from Android settings or by holding both volume keys, then explore your UI with touch to ensure nodes, labels, and actions surface correctly.
- Use `adb logcat` to capture `accesskit` logs while interacting with TalkBack if you need to confirm event flow.
