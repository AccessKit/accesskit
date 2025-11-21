# AccessKit winit adapter

This crate wires [AccessKit](https://accesskit.dev/) into [winit](https://crates.io/crates/winit) so the same code exports native accessibility trees on every supported platform.

## Adding the dependency

```toml
[dependencies]
accesskit = "0.21"
accesskit_winit = "0.29"
# winit itself chooses the window-system back ends you need.
winit = { version = "0.30", default-features = false, features = ["x11", "wayland"] }
```

Platform-specific adapter crates such as `accesskit_windows` are pulled in automatically via conditional dependencies.

### Async runtimes on Unix

Linux/*BSD builds spawn async tasks to talk to AT-SPI. Enable exactly one of the mutually exclusive features:

- Default `async-io`: uses `async-executor` and works when you are not otherwise using Tokio.
- `tokio`: reuse your application’s Tokio runtime.

## Choosing a constructor

You must create the adapter before `Window::set_visible(true)` so AccessKit can intercept the very first accessibility queries.

| Constructor | When to use it |
| --- | --- |
| [`Adapter::with_event_loop_proxy`](../src/lib.rs#L134) | Simplest path. The adapter posts `WindowEvent`s via a `winit::event_loop::EventLoopProxy`, so your `user_event` handler responds asynchronously (the initial tree must be sent later through `update_if_active`). |
| [`Adapter::with_direct_handlers`](../src/lib.rs#L178) | Provide your own `ActivationHandler`, `ActionHandler`, and `DeactivationHandler`. Use this when you can synchronously return the initial tree before showing the window. |
| [`Adapter::with_mixed_handlers`](../src/lib.rs#L214) | Hybrid approach: activation handled directly (so you can return the initial tree immediately) while action/deactivation requests still flow through the event loop proxy. |

All constructors panic if the window is already visible.

## Event-loop integration checklist

1. Create the window as invisible (`WindowAttributes::with_visible(false)`), build the adapter, then call `window.set_visible(true)`.
2. Store the adapter alongside the window state. For every winit `WindowEvent`, call [`adapter.process_event`](../src/lib.rs#L244) before handling the event yourself.
3. Handle `AccessKitEvent`s in your `user_event` callback. Respond to:
   - `InitialTreeRequested` by calling [`adapter.update_if_active`](../src/lib.rs#L258) with a closure that returns a full `TreeUpdate`.
   - `ActionRequested` by performing the requested action in your UI and sending the resulting incremental update via `update_if_active`.
   - `AccessibilityDeactivated` (usually no-op unless you want to pause expensive tracking).
4. Whenever your UI mutates (focus changes, content updates), call `update_if_active` again with only the nodes that changed and the current focus.
5. On Unix/Linux, enable exactly one of the async features as mentioned above. On Android, ensure your app uses GameActivity (see below).

The examples in [`examples/simple.rs`](examples/simple.rs) and [`examples/mixed_handlers.rs`](examples/mixed_handlers.rs) demonstrate both the event-loop-proxy workflow and the mixed handler approach.

## Android activity compatibility

The Android implementation currently works only with [GameActivity](https://developer.android.com/games/agdk/game-activity), which is one of the two activity types winit supports. If your project uses NativeActivity, you can still compile the crate but accessibility will be disabled.

## Testing

- Windows: Narrator (`Win + Ctrl + Enter`), or Accessibility Insights.
- macOS: VoiceOver (`Cmd + F5`).
- Linux: Orca (`Super + Alt + S`) and Accerciser/KDE Accessibility Inspector for tree inspection.
- Android: TalkBack (enable from Accessibility settings).

The sample apps log the key events they process, which can help you verify that your `user_event` handler is wired correctly.
