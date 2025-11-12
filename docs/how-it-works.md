# How AccessKit Works

AccessKit provides a cross-platform abstraction over native accessibility APIs, so a UI toolkit can describe its UI tree once and let AccessKit expose that tree to every supported operating system. This document is the canonical source for the “How it works” narrative and a practical quick start.

## Architecture at a glance

### Data schema

At the heart of AccessKit is a schema that expresses an accessibility tree. Each [`Node`](../common/src/lib.rs) has a stable [`NodeId`], a [`Role`] (for example `Button`, `Label`, or `TextInput`), optional attributes, and the set of actions it and its children support. Trees are delivered as [`TreeUpdate`] structs so the toolkit can stream incremental changes instead of rebuilding the entire tree.

### Tree updates and actions

The UI process (“provider”) pushes an initial tree to AccessKit and follows up with smaller updates whenever nodes change. Assistive technologies request interactions such as focus changes or button invocations through AccessKit actions; the toolkit responds by mutating its state and emitting another `TreeUpdate`. Only the platform adapter keeps a complete copy of the tree, which makes this model friendly to immediate-mode toolkits that can re-create nodes on demand as long as IDs stay stable.

### Platform adapters

Platform adapters translate the shared schema to the host accessibility API:

- `accesskit_android` (Android accessibility bridge)
- `accesskit_macos` (AppKit `NSAccessibility`)
- `accesskit_unix` (AT-SPI over D-Bus)
- `accesskit_windows` (UI Automation)

Adapters are primarily written in Rust. The [winit adapter](../platforms/winit) wraps the appropriate per-platform adapter so Rust applications using the popular windowing library only need to depend on one crate. Planned adapters include iOS and a web/canvas bridge.

### Language bindings

In addition to the Rust API, AccessKit publishes C and Python bindings. Each binding exposes the same schema (generated from the canonical Rust definitions) and forwards platform events to the native adapters.

### Consumer library

Shared, platform-independent logic that adapters need lives in the [`accesskit_consumer`](../consumer) crate. Advanced use cases—such as embedded assistive technologies that run inside a game or appliance—can build on this crate directly.

## Quick Start (Rust + winit)

The snippet below mirrors the runnable sample in [`platforms/winit/examples/simple.rs`](../platforms/winit/examples/simple.rs) and shows the pieces you need in every application: describe the tree, initialize the adapter before the window becomes visible, forward window events, and react to AccessKit requests.

### 1. Add the dependencies

```toml
[dependencies]
accesskit = "0.21"
accesskit_winit = "0.29"
# Pick the winit backend features your app needs.
winit = { version = "0.30", default-features = false, features = ["x11", "wayland"] }
```

### 2. Describe your UI tree

```rust
use accesskit::{Action, Node, NodeId, Role, Tree, TreeUpdate};

const WINDOW_ID: NodeId = NodeId(0);
const BUTTON_ID: NodeId = NodeId(1);

fn build_initial_tree() -> TreeUpdate {
    let mut window = Node::new(Role::Window);
    window.set_children(vec![BUTTON_ID]);
    window.set_label("AccessKit demo");

    let mut button = Node::new(Role::Button);
    button.set_label("Press me");
    button.add_action(Action::Click);

    TreeUpdate {
        nodes: vec![(WINDOW_ID, window), (BUTTON_ID, button)],
        tree: Some(Tree::new(WINDOW_ID)),
        focus: BUTTON_ID,
    }
}

fn button_pressed_update() -> TreeUpdate {
    let mut button = Node::new(Role::Button);
    button.set_label("Clicked!");
    button.add_action(Action::Click);

    TreeUpdate {
        nodes: vec![(BUTTON_ID, button)],
        tree: None,
        focus: BUTTON_ID,
    }
}
```

### 3. Create the adapter before showing your window

```rust
use accesskit_winit::{Adapter, Event as AccessKitEvent, WindowEvent as AccessKitWindowEvent};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::{Window, WindowId},
};

struct DemoApp {
    window: Option<Window>,
    adapter: Option<Adapter>,
    proxy: EventLoopProxy<AccessKitEvent>,
}

impl ApplicationHandler<AccessKitEvent> for DemoApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title("AccessKit demo")
            .with_visible(false);
        let window = event_loop.create_window(attrs).expect("window");
        let adapter = Adapter::with_event_loop_proxy(event_loop, &window, self.proxy.clone());
        window.set_visible(true);
        self.window = Some(window);
        self.adapter = Some(adapter);
    }

    fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if let (Some(window), Some(adapter)) = (&self.window, &mut self.adapter) {
            adapter.process_event(window, &event);
        }
    }

    fn user_event(&mut self, _: &ActiveEventLoop, event: AccessKitEvent) {
        use accesskit::{Action, ActionRequest};

        let adapter = match &mut self.adapter {
            Some(adapter) => adapter,
            None => return,
        };

        match event.window_event {
            AccessKitWindowEvent::InitialTreeRequested => {
                adapter.update_if_active(build_initial_tree);
            }
            AccessKitWindowEvent::ActionRequested(ActionRequest { action, target, .. })
                if target == BUTTON_ID && action == Action::Click =>
            {
                adapter.update_if_active(button_pressed_update);
            }
            AccessKitWindowEvent::AccessibilityDeactivated => {}
            _ => {}
        }
    }
}

fn main() -> winit::error::EventLoopError {
    let event_loop = EventLoop::with_user_event().build()?;
    let proxy = event_loop.create_proxy();
    let mut app = DemoApp {
        window: None,
        adapter: None,
        proxy,
    };
    event_loop.run_app(&mut app)
}
```

### 4. Keep tree updates incremental

- Call `adapter.process_event` for every winit `WindowEvent` so AccessKit can keep platform state in sync.
- Whenever your UI changes, call `adapter.update_if_active` with a closure that returns only the nodes that changed plus the latest focus target.
- Provide a full tree in the first update after `InitialTreeRequested`. Subsequent updates can set `tree: None`.

For more advanced scenarios (direct handlers, custom runtimes, or adapters outside winit), start from this document and explore the crate-level documentation on [docs.rs](https://docs.rs/accesskit/).
