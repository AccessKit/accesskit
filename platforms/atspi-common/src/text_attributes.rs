// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{Color, TextAlign, TextDecorationStyle};
use accesskit_consumer::Node;
use phf::phf_map;

fn color_to_string(color: Color) -> String {
    format!("{},{},{}", color.red, color.green, color.blue)
}

fn family_name(node: &Node) -> Option<String> {
    node.font_family().map(String::from)
}

fn size(node: &Node) -> Option<String> {
    node.font_size().map(|value| value.to_string())
}

fn weight(node: &Node) -> Option<String> {
    node.font_weight().map(|value| value.to_string())
}

fn style(node: &Node) -> Option<String> {
    node.is_italic().then(|| "italic".into())
}

fn strikethrough(node: &Node) -> Option<String> {
    node.strikethrough().map(|_| "true".into())
}

fn underline(node: &Node) -> Option<String> {
    node.underline().map(|deco| {
        match deco.style {
            TextDecorationStyle::Double => "double",
            _ => "single",
        }
        .into()
    })
}

fn bg_color(node: &Node) -> Option<String> {
    node.background_color().map(color_to_string)
}

fn fg_color(node: &Node) -> Option<String> {
    node.foreground_color().map(color_to_string)
}

fn language(node: &Node) -> Option<String> {
    node.language().map(String::from)
}

fn justification(node: &Node) -> Option<String> {
    node.text_align().map(|align| {
        match align {
            TextAlign::Left => "left",
            TextAlign::Center => "center",
            TextAlign::Right => "right",
            TextAlign::Justify => "fill",
        }
        .into()
    })
}

fn invalid(node: &Node) -> Option<String> {
    if node.is_spelling_error() {
        Some("spelling".into())
    } else if node.is_grammar_error() {
        Some("grammar".into())
    } else {
        None
    }
}

pub(crate) const ATTRIBUTE_GETTERS: phf::Map<&'static str, fn(&Node) -> Option<String>> = phf_map! {
    "family-name" => family_name,
    "size" => size,
    "weight" => weight,
    "style" => style,
    "strikethrough" => strikethrough,
    "underline" => underline,
    "bg-color" => bg_color,
    "fg-color" => fg_color,
    "language" => language,
    "justification" => justification,
    "invalid" => invalid,
};

#[cfg(test)]
mod tests {
    use accesskit::{Node, NodeId, Role, Tree as TreeData, TreeId, TreeUpdate};
    use accesskit_consumer::Tree;

    // A text run flagged `is_spelling_error` / `is_grammar_error` should report
    // the AT-SPI `invalid` text attribute with the matching value; an unflagged
    // run reports nothing.
    #[test]
    fn invalid_attribute_reflects_error_flags() {
        fn run(id: u64, mark: impl FnOnce(&mut Node)) -> (NodeId, Node) {
            let mut node = Node::new(Role::TextRun);
            node.set_value("word");
            node.set_character_lengths([1, 1, 1, 1]);
            mark(&mut node);
            (NodeId(id), node)
        }
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut root = Node::new(Role::TextInput);
                    root.set_children(vec![NodeId(1), NodeId(2), NodeId(3)]);
                    root
                }),
                run(1, |n| n.set_is_spelling_error()),
                run(2, |n| n.set_is_grammar_error()),
                run(3, |_| {}),
            ],
            tree: Some(TreeData::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let tree = Tree::new(update, false);
        let state = tree.state();
        let runs: Vec<_> = state.root().children().collect();

        assert_eq!(super::invalid(&runs[0]).as_deref(), Some("spelling"));
        assert_eq!(super::invalid(&runs[1]).as_deref(), Some("grammar"));
        assert_eq!(super::invalid(&runs[2]), None);
    }
}
