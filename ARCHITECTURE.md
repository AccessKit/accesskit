# ARCHITECTURE

AccessKit is a cross-platform, cross-language abstraction over accessibility APIs, so toolkit developers only have to implement accessibility once.

The expected userbase of AccessKit is developers who are writing a UI toolkit and want their toolkit to communicate with screen readers and other assistive technologies so that apps developed with the toolkit are accessible to a wider audience.


## Data schema

The heart of AccessKit is a data schema that defines all the data required to render an accessible UI for screen readers and other assistive technologies.

The schema represents a tree structure, in which each node is either a single UI element or an element cluster such as a window or document. Each node has an integer ID, a role (e.g. button, label, or text input), and a variety of optional attributes.

The schema also defines actions that can be requested by assistive technologies, such as moving the keyboard focus, invoking a button, or selecting text. The schema is based largely on Chromium's cross-platform accessibility abstraction.


## File structure

- `common/` is the main accesskit crate, it's where there is what you need to import accesskit. You use this crate to build a representation for your accessibility tree.
- `platforms/` is where you'll find the backends, crates that expose the constructed accessibility trees to the platform APIs.
- `consumer/` defines common code used by backends. Most accesskit users don't need to use that crate.


### `consumer/`

This folder holds the `accesskit_consumer` crate, which defines types and functions used by backend.

You're unlikely to need to look at `accesskit_consumer` unless you're writing a platform backend or a testing system for accesskit.

`accesskit_consumer::Tree` is the type that retains the accessibility tree in memory and updates it when a new `TreeUpdate` is emitted (see `accesskit` section).


### `platforms/`

Crates in the `platforms/` folder are what we call "adapters".

Adapters translate between accesskit's tree format and a given platform's accessibility API.

Adapters are best-effort implementations and may not cover all the accessibility APIs of their platform.

Anecdotally, adapters have very similar code; the main difference between platform APIs is the threading model.


## `accesskit` crate

This is the repository's, main crate, stored in the `common/` folder.

The main types exported by the crate are:

- `Node`
- `Role`
- `TreeUpdate`
- `Action`


### `Node`

A `Node` represents the frozen state of a semantic UI element.

A node can be a button, a date input, a widget group, a text run, a window, etc.

A node is defined by:

- A `Role`, which indicates how accessibility APIs should interpret the node.
- A list of properties, semantic information about the node.
- A list of actions that the node can receive.

For instance, a button will have the role `Button`, may have properties like "label", "disabled", "bounds", etc, and can receive the actions `Focus`, `Blur` and `Click`.

The exact list of which properties and actions apply to which roles is vague and undocumented, but mostly matches ARIA guidelines for equivalent roles.


### `TreeUpdate`

A `TreeUpdate` is a list of changes that applies to your accessibility tree. Its definition looks like:

```rust
pub struct TreeUpdate {
    pub nodes: Vec<(NodeId, Node)>,
    pub tree: Option<Tree>,
    pub tree_id: TreeId,
    pub focus: NodeId,
}
```

`nodes` is the important field here.

If `nodes` is an empty `Vec`, the node tree will stay unchanged. Otherwise, each included node will be updated.

You add new nodes by updating the parent with a children list that includes the id of new node, and you remove nodes by updating the parent with a children list without the if of the removed node.

`tree_id` indicates which tree is affected (see **Sub-trees** section).

The other two `TreeUpdate` fields (`tree`, `focus`) affect basic tree metadata.

(If you include a node, you have to re-specify everything, otherwise you erase all properties)


### `Action`

Each node has a set of actions that it accepts.

When you register that a node accepts certain actions, accesskit may give you `ActionRequest` values:

```rust
pub struct ActionRequest {
    pub action: Action,
    pub target_tree: TreeId,
    pub target_node: NodeId,
    pub data: Option<ActionData>,
}
```

Action requests are usually sent based on user input.

For instance, if you register a node with a `Button` role, an `"Apply"` label and that the node accepts `Action::Click`, if the user uses some voice control software and says "Click the 'Apply' button", your application will receive an `ActionRequest` with `Action::Click` targetting that node.


### Sub-trees

An app's accessibility tree can be composed of several subtrees.

The idea behind subtrees is to allow different actors to produce accessibility trees without coordinating (for example, because they're separate processes in a browser), and let the main application submit them to the adapters, which take care of combining them.

The main way this separation manifests is in namespacing: each subtree can pick whatever `NodeId`s it wants without worrying about colliding with other subtree's `NodeId`s.
`NodeId(123)` in one subtree and `NodeId(123)` in another subtree refer to completely different nodes

Subtrees are composed through "graft nodes", `Node` instances with a "tree_id" property set to the id of the subtree.

Each sub-tree has to be submitted to the adapter with a separate `TreeUpdate`. When presented to the platform's accessibility APIs, the sub trees will be stitched together and submitted as a single accessibility tree.


## Specification

Accesskit is currently sparsely documented (see accesskit#402), and some values are subject to interpretation. AccessKit is inspired by Chromium's accessibility API, which is undocumented, and by the ARIA standard.

In the short term, we plan on documenting which items match the ARIA standard exactly and which have a different or not-in-ARIA meaning.


## Testing

When working on AccesstKit or a project using it, it's important to check the user experience with an actual screen reader (or whatever assistive technology you're trying to support).

README-APPLICATION-DEVELOPERS.md gives some information about how to do it.
