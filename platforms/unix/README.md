# AccessKit Unix adapter

This crate exposes [AccessKit](https://accesskit.dev/) trees through AT-SPI on Linux and the BSDs.

## Adding the dependency

```toml
[target.'cfg(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))'.dependencies]
accesskit_unix = { version = "0.17", default-features = false, features = ["async-io"] }
```

### Choosing an executor

The adapter owns an async executor that communicates with the AT-SPI bus:

- Enable the default `async-io` feature if you are not already using Tokio (it pulls in `async-executor`, `async-channel`, etc.).
- Enable the mutually exclusive `tokio` feature when the surrounding application already ships Tokio. Only one of the two features may be active.

## Initialization checklist

1. **Create the adapter once per top-level window.** Call [`Adapter::new`](src/adapter.rs#L129) with your `ActivationHandler`, `ActionHandler`, and `DeactivationHandler`. The adapter immediately registers itself with the shared AT-SPI service.
2. **Report window bounds (X11).** If you can query outer/inner window rectangles, feed them to [`set_root_window_bounds`](src/adapter.rs#L168). This is especially helpful on X11 where the compositor cannot infer absolute positions.
3. **Keep focus state accurate.** When the window gains or loses focus, call [`update_window_focus_state`](src/adapter.rs#L214).
4. **Push tree updates.** Call [`update_if_active`](src/adapter.rs#L188) with a closure that returns your latest `TreeUpdate`. The adapter queues AT-SPI events internally; there is no `QueuedEvents` value to raise manually.
5. **Drop the adapter when closing the window.** The `Drop` implementation automatically unregisters from AT-SPI.

Internally the adapter processes AT-SPI traffic on background threads, so your handlers may be invoked from threads other than the UI thread.

## Enabling the accessibility bus

AT-SPI only delivers events when the accessibility bus is enabled. Starting a screen reader such as Orca usually toggles it automatically. For headless testing you can enable it manually:

```bash
busctl --user set-property org.a11y.Bus /org/a11y/bus org.a11y.Status IsEnabled b true
```

## Testing

- On GNOME, run `orca --setup` to configure the screen reader and `orca` to start it (Super + Alt + S toggles it on most desktops).
- KDE’s Accessibility Inspector and GNOME’s `accerciser` can inspect the exported tree but may not show live behavior for complex widgets. Always validate with a screen reader as well.
