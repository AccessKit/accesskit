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

## Example

The `examples/` directory contains a runnable `showcase` example built on `example_common` and `accesskit_ohos::Adapter`:

- `examples/showcase.rs` builds `libshowcase.a` / `libshowcase.so`, connecting `example_common::UiState` to `accesskit_ohos::Adapter`.
- `examples/showcase_app/` is a DevEco Studio / Hvigor application project (`dev.accesskit.showcase`) that hosts an ArkUI `XComponent`, links `libshowcase.a` into `libentry.so`, forwards `ArkUI_AccessibilityProvider` and keyboard/focus events to Rust, and bridges the OpenHarmony Input Method (`@kit.IMEKit`) for text inputs.

### Prerequisites

For an overview of HarmonyOS / OpenHarmony application development and toolchain setup, see the [HarmonyOS Application Development Overview](https://developer.huawei.com/consumer/en/doc/harmonyos-guides/start-overview).

To build and run the example locally, install:

1. **DevEco Studio** (API 13+ / HarmonyOS NEXT 5.0.1+), which provides the OpenHarmony SDK, NDK (`llvm/bin/*-unknown-linux-ohos-clang`), `hvigorw`, and `hdc`.
2. **Rust OpenHarmony targets**:

   ```sh
   rustup target add aarch64-unknown-linux-ohos x86_64-unknown-linux-ohos
   ```

> **Note on emulators:** While DevEco Studio includes an official emulator, the official emulator image does not currently support the accessibility subsystem. To test and verify accessibility behavior without a physical device, use the community-maintained OpenHarmony QEMU images from [`harmony-contrib/ohos-qemu`](https://github.com/harmony-contrib/ohos-qemu).

### Building the example

1. Configure the OpenHarmony NDK linker and build the `showcase` library from the repository root (`showcase_app` builds both `arm64-v8a` and `x86_64` by default):

   ```sh
   export OHOS_NDK_HOME="/path/to/openharmony/native"
   export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_OHOS_LINKER="$OHOS_NDK_HOME/llvm/bin/aarch64-unknown-linux-ohos-clang"
   export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_OHOS_AR="$OHOS_NDK_HOME/llvm/bin/llvm-ar"
   export CARGO_TARGET_X86_64_UNKNOWN_LINUX_OHOS_LINKER="$OHOS_NDK_HOME/llvm/bin/x86_64-unknown-linux-ohos-clang"
   export CARGO_TARGET_X86_64_UNKNOWN_LINUX_OHOS_AR="$OHOS_NDK_HOME/llvm/bin/llvm-ar"

   cargo build -p accesskit_ohos --example showcase --target aarch64-unknown-linux-ohos
   cargo build -p accesskit_ohos --example showcase --target x86_64-unknown-linux-ohos
   ```

   Alternatively, you can build with [`ohrs`](https://ohrs.org/):

   ```sh
   cargo install ohrs
   ohrs build -p accesskit_ohos -- --example showcase
   ```

2. Build the unsigned HAP package (`entry/build/default/outputs/default/entry-default-unsigned.hap`):

   - **In DevEco Studio:** Open `adapters/ohos/examples/showcase_app` and select **Build > Build Hap(s)/APP(s) > Build Hap(s)**.
   - **From the command line:**

     ```sh
     cd adapters/ohos/examples/showcase_app
     hvigorw default@PackageHap --mode module -p module=entry@default --no-daemon
     ```

### Signing the HAP

OpenHarmony / HarmonyOS NEXT requires all `.hap` packages to be signed before installation:

- **In DevEco Studio (physical device or official emulator):** Open **File > Project Structure > Signing Configs**, check **Automatically generate signature**, sign in with a developer account, and click **OK**. Running the `entry` target from DevEco Studio will automatically produce and install a signed HAP.
- **From the command line (OpenHarmony QEMU):** You can sign `entry-default-unsigned.hap` with the OpenHarmony self-signed root certificates using [`hapsigner-rs` (`hap-sign`)](https://github.com/harmony-contrib/hapsigner) or the SDK's `hap-sign-tool.jar`:

  ```sh
  # Install hap-sign
  cargo install --git https://github.com/harmony-contrib/hapsigner

  # Sign the unsigned HAP for dev.accesskit.showcase
  hap-sign sign \
    adapters/ohos/examples/showcase_app/entry/build/default/outputs/default/entry-default-unsigned.hap \
    --output showcase-signed.hap \
    --bundle-name dev.accesskit.showcase \
    --force
  ```

### Running and verifying on OpenHarmony QEMU

1. Download and start an OpenHarmony QEMU image from [`harmony-contrib/ohos-qemu`](https://github.com/harmony-contrib/ohos-qemu) (for example, `openharmony-qemu-arm64-arm64_virt-2in1`):

   ```sh
   ./start.sh --headless --port 5558
   ```

2. Connect `hdc` to the QEMU instance, install the signed HAP, and launch `EntryAbility`:

   ```sh
   hdc tconn 127.0.0.1:5558
   hdc -t 127.0.0.1:5558 install showcase-signed.hap
   hdc -t 127.0.0.1:5558 shell aa start -b dev.accesskit.showcase -a EntryAbility
   ```

3. Verify the accessibility tree exported by `accesskit_ohos` through the system accessibility framework using `uitest dumpLayout`:

   ```sh
   hdc -t 127.0.0.1:5558 shell uitest dumpLayout -p /data/local/tmp/showcase-layout.json
   hdc -t 127.0.0.1:5558 file recv /data/local/tmp/showcase-layout.json ./showcase-layout.json
   ```

   In `showcase-layout.json`, the `XComponent` node (`"type": "XComponent"`) will contain the child accessibility tree (`treeId: 1`) populated by `accesskit_ohos`, including the login screen image, labels, focused email `textInput` (`"hint": "name@company.com"`), password `textInput`, checkbox, and button nodes with their screen coordinates.

### Validating with ScreenReader on a physical device

1. On a physical OpenHarmony / HarmonyOS NEXT device, enable **Settings > Accessibility > ScreenReader** (屏幕朗读).
2. Launch **AccessKit Showcase**.
3. Swipe left and right to move accessibility focus through the showcase widgets (labels, text inputs, password visibility checkbox, login button, chat contact list, and message list).
4. Double-tap to activate buttons, toggle checkboxes, or focus text inputs and bring up the on-screen keyboard.
5. On scrollable containers, swipe with two fingers (or swipe up/down) to test scroll actions and live-region message announcements.

## License

MIT OR Apache-2.0
