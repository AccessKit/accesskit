# AccessKit Windows adapter

This crate bridges [AccessKit](https://accesskit.dev/) trees to the Windows UI Automation (UIA) API.

## Adding the dependency

```toml
[target.'cfg(target_os = "windows")'.dependencies]
accesskit_windows = "0.29"
```

## Initialization checklist

1. **Create the adapter before UIA asks for the window.** Call [`Adapter::new`](src/adapter.rs) as soon as the window handle is available and before the first `WM_GETOBJECT` message can be delivered. Pass the HWND, the current focus state, and an `ActionHandler` implementation.
2. **Supply an `ActivationHandler`.** Your window procedure should handle `WM_GETOBJECT` by calling [`adapter.handle_wm_getobject`](src/adapter.rs#L454) with the message parameters and the activation handler. Return the provided `LRESULT` if AccessKit handled the message; otherwise fall back to `DefWindowProcW`.
3. **Forward focus changes.** On `WM_SETFOCUS`, `WM_KILLFOCUS`, modal loops, etc., call [`adapter.update_window_focus_state`](src/adapter.rs#L399). This keeps screen readers in sync with the real focus.
4. **Push tree updates.** Whenever your UI changes, call [`adapter.update_if_active`](src/adapter.rs#L358) with a closure that produces a `TreeUpdate`. If it returns [`QueuedEvents`](src/adapter.rs#L549), drop any locks and invoke `raise()` on the thread that owns the window.
5. **Raise events outside window locks.** `QueuedEvents::raise` can synchronously trigger more `WM_GETOBJECT` traffic, so never call it while holding locks that your window procedure depends on (see `platforms/windows/examples/hello_world.rs` for a working pattern).

If your UI toolkit does not expose its own window procedure, use [`SubclassingAdapter`](src/subclass.rs) to install a Win32 subclass that wires the steps above automatically. Create it before showing the window so it can intercept `WM_GETOBJECT`.

## Threading & testing

- `Adapter::update_if_active` and `update_window_focus_state` are thread-safe, but `QueuedEvents::raise` must run on the UI thread that owns the HWND.
- UIA is only available on Windows 7+; AccessKit targets Windows 10 and later where UIA is the primary accessibility API.
- Use Narrator (`Win + Ctrl + Enter`) or the Accessibility Insights tool to verify events. The sample in [`platforms/windows/examples/hello_world.rs`](examples/hello_world.rs) exercises focus changes, button clicks, and live regions.
