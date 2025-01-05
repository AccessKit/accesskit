use accesskit::{Action, ActionRequest, Node, NodeId, Rect, Role, TextPosition, TextSelection, Toggled, TreeUpdate};

use crate::{group, node_id, tabs::Tab, Key, Widget, CHARACTER_HEIGHT, CHARACTER_WIDTH, MARGIN, PADDING};

//const WIDGET_HEIGHT: f64 = CHARACTER_HEIGHT + PADDING * 2.0;

pub(crate) struct TextTab {
    has_focus: bool,
    focus: NodeId,

    input_id: NodeId,

    lines: Vec<String>,
    selection_start_line: usize,
    selection_start_offset: usize,
    selection_end_line: usize,
    selection_end_offset: usize,
}

impl TextTab {
    pub(crate) fn new() -> Self {
        let input_id = node_id!();
        Self {
            has_focus: false,
            focus: input_id,
            input_id,
            lines: vec![
                "AccessKit Text Input\n".to_string(),
                "\n".to_string(),
                "This is the last line.".to_string(),
            ],
            selection_start_line: 0,
            selection_start_offset: 0,
            selection_end_line: 0,
            selection_end_offset: 0,
        }
    }
}

impl Widget for TextTab {
    fn key_pressed(&mut self, key: Key) -> bool {
        match key {
            Key::Left => {
                if self.selection_start_offset > 0 {
                    self.selection_start_offset -= 1;
                } else if self.selection_start_line > 0 {
                    self.selection_start_line -= 1;
                    let current_line_length = self.lines[self.selection_start_line].chars().count();
                    self.selection_start_offset = current_line_length - 1;
                }
            }
            Key::Right => {
                let current_line_length = self.lines[self.selection_start_line].chars().count();
                if self.selection_start_offset < current_line_length - 1 {
                    self.selection_start_offset += 1;
                } else if self.selection_start_line < self.lines.len() - 1 {
                    self.selection_start_line += 1;
                    self.selection_start_offset = 0;
                } else {
                    self.selection_start_offset = current_line_length;
                }
            }
            _ => (),
        }
        self.selection_end_line = self.selection_start_line;
        self.selection_end_offset = self.selection_start_offset;
        true
    }

    fn has_focus(&self) -> bool {
        self.has_focus
    }

    fn set_focused(&mut self, has_focus: bool) {
        self.has_focus = has_focus;
    }

    fn render(&self, update: &mut TreeUpdate) -> NodeId {
        let text_run_ids = (0..self.lines.len()).map(|i| node_id!("", i)).collect::<Vec<NodeId>>();
        let mut input = Node::new(Role::MultilineTextInput);
        input.set_children(text_run_ids.clone());
        let selection = TextSelection {
            anchor: TextPosition {
                node: text_run_ids[self.selection_start_line],
                character_index: self.selection_start_offset,
            },
            focus: TextPosition {
                node: text_run_ids[self.selection_end_line],
                character_index: self.selection_end_offset,
            },
        };
        input.set_text_selection(selection);
        update.nodes.push((self.input_id, input));
        for (i, line) in self.lines.iter().enumerate() {
            let mut run = Node::new(Role::TextRun);
            run.set_character_lengths(line.chars().map(|c| c.len_utf8() as u8).collect::<Vec<u8>>());
            run.set_character_positions((0..line.len()).map(|i| i as f32 * CHARACTER_WIDTH as f32).collect::<Vec<f32>>());
            run.set_character_widths((0..line.len()).map(|_| CHARACTER_WIDTH as f32).collect::<Vec<f32>>());
            run.set_value(line.as_str());
            update.nodes.push((text_run_ids[i], run));
        }
        if self.has_focus() {
            update.focus = self.focus;
        }
        self.input_id
    }

    fn do_action(&mut self, request: &ActionRequest) -> bool {
        false
    }
}

impl Tab for TextTab {
    fn title(&self) -> &str {
        "Text Input"
    }
}
