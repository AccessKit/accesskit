# AccessKit macOS adapter

This crate exposes [AccessKit](https://accesskit.dev/) trees through AppKit's `NSAccessibility` protocol.

## Adding the dependency

```toml
[target.'cfg(target_os = "macos")'.dependencies]
accesskit_macos = "0.22"
```

## Usage patterns

### Integrate with your own `NSView`

1. **Create the adapter on the main thread.** Call `unsafe { Adapter::new(view_ptr, is_focused, action_handler) }` where `view_ptr` is the `NSView*`. The action handler is always invoked on the main thread.
2. **Keep an `ActivationHandler` near the adapter.** Pass it to the methods that query the tree (`view_children`, `focus`, and `hit_test`) so the adapter can lazily request the initial tree when VoiceOver first inspects the view.
3. **Override accessibility entry points.** In your Objective-C/Swift subclass, forward `accessibilityChildren`, `accessibilityFocusedUIElement`, and `accessibilityHitTest:` to the adapter helper methods above, and return the bridged `NSArray`/`NSObject` pointers they produce.
4. **Push updates.** Whenever the UI changes, call `adapter.update_if_active` with a closure that returns a `TreeUpdate`. If it produces [`QueuedEvents`](src/event.rs#L110), call `raise()` once you have dropped any locks; VoiceOver may re-enter your accessibility overrides while events are raised.
5. **Track focus state.** On `NSWindow` focus changes, call `adapter.update_view_focus_state`.

See [`platforms/winit/examples/simple.rs`](../winit/examples/simple.rs) for a full Rust example that forwards these calls from winit’s Cocoa backend.

### Use the dynamic subclass helper

If you cannot subclass the `NSView` yourself (e.g., when integrating with SDL), use [`SubclassingAdapter`](src/subclass.rs). It dynamically swaps in a generated Objective-C subclass that overrides the necessary accessibility methods, while you provide the `ActivationHandler` and `ActionHandler`.

### Forwarding `NSWindow` focus

Some toolkits place keyboard focus directly on the `NSWindow`. Call [`add_focus_forwarder_to_window_class`](src/patch.rs) once at startup to wire the window’s `accessibilityFocusedUIElement` to its content view. This helper modifies the Objective-C class at runtime, so prefer linking AccessKit statically into the main binary.

## Testing

- VoiceOver (`Cmd + F5`) is the reference screen reader on macOS.
- The Xcode Accessibility Inspector is useful for quick tree inspections but can miss screen-reader-specific behavior.

## Known issues

- The selected state of ListBox items is not reported ([#520](https://github.com/AccessKit/accesskit/issues/520)).
