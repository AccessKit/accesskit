// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::Color;
use accesskit_consumer::Node;
use phf::phf_map;

fn color_to_string(color: Color) -> String {
    format!("{},{},{}", color.red, color.green, color.blue)
}

fn family_name(node: &Node) -> Option<String> {
    node.font_family().map(String::from)
}

fn bg_color(node: &Node) -> Option<String> {
    node.background_color().map(color_to_string)
}

fn fg_color(node: &Node) -> Option<String> {
    node.foreground_color().map(color_to_string)
}

pub(crate) const ATTRIBUTE_GETTERS: phf::Map<&'static str, fn(&Node) -> Option<String>> = phf_map! {
    "family-name" => family_name,
    "bg-color" => bg_color,
    "fg-color" => fg_color,
};
