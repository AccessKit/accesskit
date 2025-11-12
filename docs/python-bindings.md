# AccessKit from Python

The [`accesskit`](https://pypi.org/project/accesskit/) Python package exposes the AccessKit schema and platform adapters via [PyO3](https://pyo3.rs/). Use it when building games or custom toolkits in Python that still need first-class accessibility support.

## Installation

```bash
python -m pip install --upgrade accesskit
```

Prebuilt wheels are published for the most common CPython versions on Windows, macOS, and Linux. If your platform has no wheel, `pip` falls back to building the Rust sources locally, so ensure `rustup` is installed.

## Basic usage pattern

```python
from accesskit import Node, NodeId, Role, Tree, TreeUpdate
from accesskit.adapters.win32 import Adapter  # or .macos / .unix / .android

WINDOW_ID = NodeId(0)
BUTTON_ID = NodeId(1)

def initial_tree():
    window = Node.new(Role.Window)
    window.set_children([BUTTON_ID])
    button = Node.new(Role.Button)
    button.set_label("Press me")
    button.add_default_action()
    return TreeUpdate(
        nodes=[(WINDOW_ID, window), (BUTTON_ID, button)],
        tree=Tree.new(WINDOW_ID),
        focus=BUTTON_ID,
    )

adapter = Adapter(hwnd=...)  # platform-specific handle
adapter.update(initial_tree())
```

Each adapter module mirrors the Rust API: you supply activation/action handlers, pump events from your GUI toolkit, and call `update` whenever your UI changes. The [pygame example](https://github.com/AccessKit/accesskit-python/tree/main/examples/pygame) in the Python repository demonstrates a full event loop that sends tree updates and responds to action requests.

## Building from source

If you need to build wheels yourself (e.g., for CI):

```bash
python -m pip install maturin
maturin build --release
```

Run these commands inside the `accesskit-python` repository. Maturin handles compiling the Rust code and packaging the wheel for your current interpreter/target.

## Contributions and support

- Source: <https://github.com/AccessKit/accesskit-python>
- Issues: <https://github.com/AccessKit/accesskit-python/issues>
- Docs for the underlying Rust crates: <https://docs.rs/accesskit/>
