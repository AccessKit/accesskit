# accesskit_ohos

AccessKit adapter for the ArkUI native accessibility provider API on
OpenHarmony/HarmonyOS NEXT.

The adapter is attached to an accessibility `Provider` obtained from an
XComponent or an `ARKUI_NODE_CUSTOM`. It retains the AccessKit tree, answers
ArkUI's synchronous node queries, forwards native actions to AccessKit, and
reports tree/focus changes back to ArkUI.

```rust,ignore
let provider = xcomponent.accessibility_provider()?;
let adapter = accesskit_ohos::Adapter::new(
    provider,
    activation_handler,
    action_handler,
)?;

adapter.update_if_active(|| next_tree_update());
```

`Adapter::new` uses the standard callback registration for a single ArkUI custom
node or XComponent. The binding permits only one such registration per process.
For multiple XComponent windows or recreated surfaces, enable `api-15` and use
`Adapter::new_with_instance` with a unique ID for each registration, so late
callbacks cannot reach another window or a replacement surface's tree.

Drop the adapter before destroying the XComponent or custom node that owns its
provider.

The OpenHarmony native API does not expose an unregister function. Dropping
the adapter disables its Rust callback target, so any late native callback
returns a failure without touching released application state.

Action descriptions use static C strings because ArkUI snapshots retain their
description pointers. Event submission keeps its attached element alive through
the binding's borrowed event lifetime.

## API levels

The adapter defaults to the `api-13` feature, which is sufficient for an
XComponent accessibility provider and for all core tree, action, and event
operations. Higher API features form an additive ladder:

- API 15 adds multi-instance callback registration and next/previous HTML item
  action handling.
- An ArkUI `ARKUI_NODE_CUSTOM` host needs API 23 to obtain its accessibility
  provider. This requirement belongs to the ArkUI host integration; the
  adapter still only consumes the resulting `Provider`.
- Enable `api-24` when AccessKit `author_id` values must be exposed through
  ArkUI's `componentIdentifier` property.

Applications should select the highest API feature supported by their minimum
deployment target. Enabling `api-24` is not required for the core adapter.

## License

MIT OR Apache-2.0
