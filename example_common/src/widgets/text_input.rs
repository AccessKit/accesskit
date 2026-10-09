use accesskit::{
    Action, ActionData, ActionRequest, Node, NodeId, Rect, Role, TextDirection, TextPosition,
    TextSelection,
};
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

use super::{Response, Widget};
use crate::{Key, KeyEvent, KeyState};

/// Text runs identify word starts with `u8` indices, so a line longer than
/// this is split into several runs, linked to each other as being on the
/// same line.
const MAX_RUN_LENGTH: usize = u8::MAX as usize + 1;

const PADDING: f64 = 12.0;
/// The width of every character, as if the text were drawn in a monospace font.
const CHARACTER_WIDTH: f32 = 9.0;
const LINE_HEIGHT: f64 = 20.0;
/// The character masking those of a password, the one each platform's own
/// password fields use.
const PASSWORD_MASK: &str = if cfg!(any(
    target_os = "android",
    target_os = "ios",
    target_os = "macos"
)) {
    "\u{2022}"
} else {
    "\u{25cf}"
};

/// The kind of text a [`TextInput`] accepts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputType {
    /// A single line of text.
    Text,
    /// An email address.
    Email,
    /// A password, which is masked.
    Password,
    /// Text that wraps, and may contain line breaks.
    Multiline,
}

impl InputType {
    fn role(self) -> Role {
        match self {
            Self::Text => Role::TextInput,
            Self::Email => Role::EmailInput,
            Self::Password => Role::PasswordInput,
            Self::Multiline => Role::MultilineTextInput,
        }
    }
}

/// The text of a [`TextInput`], its selection, and the text being composed
/// by an input method.
///
/// Positions are byte offsets into the text. A character is a grapheme
/// cluster, and an input method may put positions inside one while it
/// composes it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextInputState {
    text: String,
    anchor: usize,
    focus: usize,
    composition: Option<Range<usize>>,
}

/// The state of a [`TextInput`], for the platform's input method.
///
/// Positions are byte offsets into `text`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextInputInfo {
    pub id: NodeId,
    /// The kind of text the input accepts.
    pub input_type: InputType,
    /// Whether submitting the text does anything.
    pub can_submit: bool,
    pub text: String,
    /// Where the selection starts.
    pub selection_anchor: usize,
    /// Where the selection ends, and the caret is.
    pub selection_focus: usize,
    /// The text being composed, if any.
    pub composition: Option<Range<usize>>,
}

/// An edit from the platform's input method.
///
/// Positions are byte offsets into the text, rounded down to code point
/// boundaries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImeEdit {
    /// Replaces the text in `range`. The selection and the text being
    /// composed move along with the text around them.
    Replace {
        range: Range<usize>,
        text: String,
    },
    /// Marks text as being composed, or ends the composition.
    SetComposition(Option<Range<usize>>),
    SetSelection {
        anchor: usize,
        focus: usize,
    },
    /// Submits the text, like the enter key of a single line input.
    Submit,
}

impl TextInputState {
    pub fn text(&self) -> &str {
        &self.text
    }

    fn code_point_boundary(&self, offset: usize) -> usize {
        let mut offset = offset.min(self.text.len());
        while !self.text.is_char_boundary(offset) {
            offset -= 1;
        }
        offset
    }

    fn code_point_range(&self, range: &Range<usize>) -> Range<usize> {
        let start = self.code_point_boundary(range.start);
        let end = self.code_point_boundary(range.end);
        start.min(end)..start.max(end)
    }

    fn previous_character_start(&self, offset: usize) -> usize {
        self.text
            .grapheme_indices(true)
            .map(|(start, _)| start)
            .take_while(|start| *start < offset)
            .last()
            .unwrap_or(0)
    }

    fn next_character_end(&self, offset: usize) -> usize {
        self.text
            .grapheme_indices(true)
            .map(|(start, character)| start + character.len())
            .find(|end| *end > offset)
            .unwrap_or(self.text.len())
    }

    fn replace(&mut self, range: &Range<usize>, text: &str) {
        let range = self.code_point_range(range);
        let map = |offset: usize| {
            if offset <= range.start {
                offset
            } else if offset >= range.end {
                offset - range.len() + text.len()
            } else {
                range.start
            }
        };
        self.text.replace_range(range.clone(), text);
        self.anchor = map(self.anchor);
        self.focus = map(self.focus);
        self.composition = self
            .composition
            .take()
            .map(|composition| map(composition.start)..map(composition.end))
            .filter(|composition| !composition.is_empty());
    }

    fn apply_ime_edit(&mut self, edit: &ImeEdit) {
        match edit {
            ImeEdit::Replace { range, text } => self.replace(range, text),
            ImeEdit::SetComposition(composition) => {
                self.composition = composition
                    .as_ref()
                    .map(|composition| self.code_point_range(composition))
                    .filter(|composition| !composition.is_empty());
            }
            ImeEdit::SetSelection { anchor, focus } => self.select(*anchor, *focus),
            ImeEdit::Submit => (),
        }
    }

    fn selection(&self) -> Range<usize> {
        self.anchor.min(self.focus)..self.anchor.max(self.focus)
    }

    fn select(&mut self, anchor: usize, focus: usize) {
        self.anchor = self.code_point_boundary(anchor);
        self.focus = self.code_point_boundary(focus);
    }

    fn replace_selection(&mut self, text: &str) {
        let selection = self.selection();
        self.replace(&selection, text);
        self.composition = None;
        let caret = selection.start + text.len();
        self.select(caret, caret);
    }

    fn set_text(&mut self, text: &str) {
        self.select(0, self.text.len());
        self.replace_selection(text);
    }

    fn delete_backward(&mut self) {
        if self.anchor == self.focus {
            self.select(self.previous_character_start(self.focus), self.focus);
        }
        self.replace_selection("");
    }

    fn delete_forward(&mut self) {
        if self.anchor == self.focus {
            self.select(self.focus, self.next_character_end(self.focus));
        }
        self.replace_selection("");
    }

    fn move_to(&mut self, offset: usize, extend: bool) {
        let anchor = if extend { self.anchor } else { offset };
        self.select(anchor, offset);
    }

    fn move_left(&mut self, extend: bool) {
        match self.selection() {
            selection if !extend && !selection.is_empty() => self.move_to(selection.start, false),
            _ => self.move_to(self.previous_character_start(self.focus), extend),
        }
    }

    fn move_right(&mut self, extend: bool) {
        match self.selection() {
            selection if !extend && !selection.is_empty() => self.move_to(selection.end, false),
            _ => self.move_to(self.next_character_end(self.focus), extend),
        }
    }
}

fn is_whitespace(character: &str) -> bool {
    character.chars().all(char::is_whitespace)
}

fn is_hard_line_break(character: &str) -> bool {
    character.ends_with('\n')
}

/// The characters of a line. A hard line break is the last character of its
/// line, while a soft line break is between two lines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Line {
    start: usize,
    end: usize,
}

/// Wraps text into lines of at most `width` characters, not counting their
/// trailing whitespace.
fn wrap(characters: &[&str], width: usize) -> Vec<Line> {
    let mut lines = Vec::new();
    let mut paragraph_start = 0;
    loop {
        let paragraph_end = characters[paragraph_start..]
            .iter()
            .position(|character| is_hard_line_break(character))
            .map(|offset| paragraph_start + offset);
        let text_end = paragraph_end.unwrap_or(characters.len());
        let mut line_start = paragraph_start;
        let mut index = paragraph_start;
        while index < text_end {
            let word_start = index;
            while index < text_end && !is_whitespace(characters[index]) {
                index += 1;
            }
            let word_end = index;
            while index < text_end && is_whitespace(characters[index]) {
                index += 1;
            }
            if word_start > line_start && word_end - line_start > width {
                lines.push(Line {
                    start: line_start,
                    end: word_start,
                });
                line_start = word_start;
            }
            while word_end - line_start > width {
                lines.push(Line {
                    start: line_start,
                    end: line_start + width,
                });
                line_start += width;
            }
        }
        match paragraph_end {
            Some(end) => {
                lines.push(Line {
                    start: line_start,
                    end: end + 1,
                });
                paragraph_start = end + 1;
            }
            None => {
                lines.push(Line {
                    start: line_start,
                    end: text_end,
                });
                return lines;
            }
        }
    }
}

/// A text input.
///
/// The text is exposed as text runs, so that assistive technologies can
/// navigate it and follow the caret. Each line has its own runs, linked to
/// each other as being on the same line when it's too long for one. The
/// runs of a password don't reveal where its words start.
pub struct TextInput<M> {
    id: NodeId,
    input_type: InputType,
    state: TextInputState,
    bounds: Rect,
    /// Where each character starts in the text, followed by the text's length.
    character_offsets: Vec<usize>,
    lines: Vec<Line>,
    text_runs: Vec<TextRun>,
    label: Option<String>,
    labelled_by: Option<NodeId>,
    placeholder: Option<String>,
    required: bool,
    disabled: bool,
    on_change: fn(TextInputState) -> M,
    on_submit: Option<M>,
}

impl<M> TextInput<M> {
    /// Creates a text input. `on_change` receives the new state.
    ///
    /// The text runs use consecutive IDs from `first_text_run_id`, as many as
    /// the text needs.
    pub fn new(
        id: NodeId,
        first_text_run_id: NodeId,
        input_type: InputType,
        state: &TextInputState,
        bounds: Rect,
        on_change: fn(TextInputState) -> M,
    ) -> Self {
        let (mut character_offsets, characters): (Vec<usize>, Vec<&str>) =
            state.text.grapheme_indices(true).unzip();
        character_offsets.push(state.text.len());
        let (characters, word_starts) = match input_type {
            InputType::Password => (vec![PASSWORD_MASK; characters.len()], Vec::new()),
            _ => {
                let word_starts = state
                    .text
                    .split_word_bound_indices()
                    .filter(|(_, word)| !is_whitespace(word))
                    .map(|(offset, _)| character_offsets.partition_point(|start| *start < offset))
                    .collect();
                (characters, word_starts)
            }
        };
        let lines = if input_type == InputType::Multiline {
            let width = (bounds.width() - 2.0 * PADDING) / CHARACTER_WIDTH as f64;
            wrap(&characters, (width as usize).max(1))
        } else {
            vec![Line {
                start: 0,
                end: characters.len(),
            }]
        };
        let mut text_runs = Vec::<TextRun>::new();
        for (line_index, line) in lines.iter().enumerate() {
            let mut start = line.start;
            loop {
                let end = (start + MAX_RUN_LENGTH).min(line.end);
                let id = NodeId(first_text_run_id.0 + text_runs.len() as u64);
                let mut run = TextRun::new(
                    id,
                    &characters,
                    &word_starts,
                    start..end,
                    *line,
                    bounds,
                    line_index,
                );
                if let Some(previous) = text_runs.last_mut().filter(|_| start > line.start) {
                    previous.next_on_line = Some(id);
                    run.previous_on_line = Some(previous.id);
                }
                text_runs.push(run);
                start = end;
                if start >= line.end {
                    break;
                }
            }
        }
        Self {
            id,
            input_type,
            state: state.clone(),
            bounds,
            character_offsets,
            lines,
            text_runs,
            label: None,
            labelled_by: None,
            placeholder: None,
            required: false,
            disabled: false,
            on_change,
            on_submit: None,
        }
    }

    /// Sets the name of the input, when it has no visible label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the visible label of the input.
    pub fn with_labelled_by(mut self, label: NodeId) -> Self {
        self.labelled_by = Some(label);
        self
    }

    /// Sets a hint about what to enter, which AccessKit only exposes while
    /// the input is empty.
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the message produced when the text is submitted with Enter. A
    /// multiline input then breaks lines with Ctrl+Enter.
    pub fn with_on_submit(mut self, message: M) -> Self {
        self.on_submit = Some(message);
        self
    }

    /// The index of the character containing the byte at `offset`, or the
    /// number of characters at the end of the text.
    fn character_index(&self, offset: usize) -> usize {
        self.character_offsets
            .partition_point(|start| *start <= offset)
            - 1
    }

    fn character_count(&self) -> usize {
        self.character_offsets.len() - 1
    }

    fn line_index(&self, index: usize) -> usize {
        self.lines
            .iter()
            .rposition(|line| line.start <= index)
            .unwrap_or(0)
    }

    fn line_end(&self, line_index: usize) -> usize {
        let end = self.lines[line_index].end;
        if line_index + 1 == self.lines.len() {
            end
        } else {
            end - 1
        }
    }

    fn vertical_move(&self, down: bool) -> usize {
        let focus = self.character_index(self.state.focus);
        let line_index = self.line_index(focus);
        let column = focus - self.lines[line_index].start;
        let target = if down {
            Some(line_index + 1).filter(|target| *target < self.lines.len())
        } else {
            line_index.checked_sub(1)
        };
        match target {
            Some(target) => (self.lines[target].start + column).min(self.line_end(target)),
            None if down => self.character_count(),
            None => 0,
        }
    }

    /// The position of the character containing the byte at `offset` in the
    /// text runs. At the boundary of two runs, that's the start of the
    /// second one.
    fn position(&self, offset: usize) -> TextPosition {
        let index = self.character_index(offset);
        let run = self
            .text_runs
            .iter()
            .rfind(|run| run.range.start <= index)
            .unwrap_or(&self.text_runs[0]);
        TextPosition {
            node: run.id,
            character_index: index - run.range.start,
        }
    }

    fn offset(&self, position: &TextPosition) -> Option<usize> {
        let run = self.text_runs.iter().find(|run| run.id == position.node)?;
        let index = run.range.start + position.character_index.min(run.range.len());
        Some(self.character_offsets[index])
    }

    fn edit(&self, edit: impl FnOnce(&mut TextInputState)) -> Response<M> {
        let mut state = self.state.clone();
        state.composition = None;
        edit(&mut state);
        self.respond(state)
    }

    fn respond(&self, state: TextInputState) -> Response<M> {
        if state == self.state {
            Response::Handled
        } else {
            Response::Message((self.on_change)(state))
        }
    }
}

impl<M: Clone> Widget<M> for TextInput<M> {
    fn id(&self) -> NodeId {
        self.id
    }

    fn build(&self) -> Node {
        let mut node = Node::new(self.input_type.role());
        node.set_bounds(self.bounds);
        node.set_children(self.text_runs.iter().map(|run| run.id).collect::<Vec<_>>());
        if let Some(label) = &self.label {
            node.set_label(label.as_str());
        }
        if let Some(label) = self.labelled_by {
            node.set_labelled_by(vec![label]);
        }
        if let Some(placeholder) = &self.placeholder {
            node.set_placeholder(placeholder.as_str());
        }
        if self.required {
            node.set_required();
        }
        node.set_text_selection(TextSelection {
            anchor: self.position(self.state.anchor),
            focus: self.position(self.state.focus),
        });
        if self.disabled {
            node.set_disabled();
        } else {
            node.add_action(Action::Focus);
            node.add_action(Action::Click);
            node.add_action(Action::SetTextSelection);
            node.add_action(Action::ReplaceSelectedText);
            node.add_action(Action::SetValue);
        }
        node
    }

    fn children(&self) -> Vec<&dyn Widget<M>> {
        self.text_runs.iter().map(|run| run as _).collect()
    }

    fn is_focusable(&self) -> bool {
        !self.disabled
    }

    fn on_key(&self, event: &KeyEvent, _focus: NodeId) -> Response<M> {
        if self.disabled || event.state != KeyState::Pressed {
            return Response::Ignored;
        }
        let is_multiline = self.input_type == InputType::Multiline;
        let extend = event.modifiers.shift;
        let line_index = self.line_index(self.character_index(self.state.focus));
        let mut state = self.state.clone();
        state.composition = None;
        match event.key {
            Key::Enter if is_multiline && (event.modifiers.control || self.on_submit.is_none()) => {
                state.replace_selection("\n")
            }
            Key::Enter => {
                return self
                    .on_submit
                    .clone()
                    .map_or(Response::Ignored, Response::Message);
            }
            Key::Tab => return Response::Ignored,
            Key::Character(c) => state.replace_selection(c.encode_utf8(&mut [0; 4])),
            Key::Space => state.replace_selection(" "),
            Key::Backspace => state.delete_backward(),
            Key::Delete => state.delete_forward(),
            Key::Left => state.move_left(extend),
            Key::Right => state.move_right(extend),
            Key::Up => state.move_to(self.character_offsets[self.vertical_move(false)], extend),
            Key::Down => state.move_to(self.character_offsets[self.vertical_move(true)], extend),
            Key::Home => {
                state.move_to(self.character_offsets[self.lines[line_index].start], extend)
            }
            Key::End => state.move_to(self.character_offsets[self.line_end(line_index)], extend),
        }
        self.respond(state)
    }

    fn on_action(&self, request: &ActionRequest) -> Response<M> {
        if self.disabled {
            return Response::Ignored;
        }
        match (request.action, &request.data) {
            (Action::Focus | Action::Click, _) => Response::Focus(self.id),
            (Action::SetTextSelection, Some(ActionData::SetTextSelection(selection))) => {
                let (Some(anchor), Some(focus)) = (
                    self.offset(&selection.anchor),
                    self.offset(&selection.focus),
                ) else {
                    return Response::Ignored;
                };
                self.edit(|state| state.select(anchor, focus))
            }
            (Action::ReplaceSelectedText, Some(ActionData::Value(text))) => {
                self.edit(|state| state.replace_selection(text))
            }
            (Action::SetValue, Some(ActionData::Value(text))) => {
                self.edit(|state| state.set_text(text))
            }
            _ => Response::Ignored,
        }
    }

    fn text_input(&self) -> Option<TextInputInfo> {
        if self.disabled {
            return None;
        }
        let state = &self.state;
        Some(TextInputInfo {
            id: self.id,
            input_type: self.input_type,
            can_submit: self.on_submit.is_some(),
            text: state.text.clone(),
            selection_anchor: state.anchor,
            selection_focus: state.focus,
            composition: state.composition.clone(),
        })
    }

    fn on_ime_edit(&self, edit: &ImeEdit) -> Response<M> {
        if self.disabled {
            return Response::Ignored;
        }
        if *edit == ImeEdit::Submit {
            return self
                .on_submit
                .clone()
                .map_or(Response::Ignored, Response::Message);
        }
        let mut state = self.state.clone();
        state.apply_ime_edit(edit);
        self.respond(state)
    }
}

/// The text of a line of a [`TextInput`], or of part of it.
///
/// A run that continues a line may start in the middle of a word. A line
/// break has no width, as in Notepad. The text direction is needed for
/// platforms to compute the bounds of characters. A character longer than
/// 255 bytes, which AccessKit can't express, isn't supported.
struct TextRun {
    id: NodeId,
    range: Range<usize>,
    bounds: Rect,
    characters: Vec<String>,
    word_starts: Vec<u8>,
    previous_on_line: Option<NodeId>,
    next_on_line: Option<NodeId>,
}

impl TextRun {
    fn new(
        id: NodeId,
        characters: &[&str],
        word_starts: &[usize],
        range: Range<usize>,
        line: Line,
        input_bounds: Rect,
        line_index: usize,
    ) -> Self {
        let run_characters = &characters[range.clone()];
        let word_starts = word_starts
            .iter()
            .filter(|start| range.contains(start))
            .map(|start| (start - range.start) as u8)
            .collect();
        let visible_count = run_characters
            .iter()
            .filter(|character| !is_hard_line_break(character))
            .count();
        let x0 = input_bounds.x0
            + PADDING
            + ((range.start - line.start) as f32 * CHARACTER_WIDTH) as f64;
        let y0 = input_bounds.y0 + PADDING + line_index as f64 * LINE_HEIGHT;
        Self {
            id,
            bounds: Rect {
                x0,
                y0,
                x1: x0 + (visible_count as f32 * CHARACTER_WIDTH) as f64,
                y1: y0 + LINE_HEIGHT,
            },
            characters: run_characters.iter().map(|c| c.to_string()).collect(),
            range,
            word_starts,
            previous_on_line: None,
            next_on_line: None,
        }
    }
}

impl<M> Widget<M> for TextRun {
    fn id(&self) -> NodeId {
        self.id
    }

    fn build(&self) -> Node {
        let mut node = Node::new(Role::TextRun);
        node.set_bounds(self.bounds);
        node.set_text_direction(TextDirection::LeftToRight);
        node.set_value(self.characters.concat());
        node.set_character_lengths(
            self.characters
                .iter()
                .map(|character| character.len() as u8)
                .collect::<Vec<_>>(),
        );
        node.set_character_positions(
            (0..self.characters.len())
                .map(|i| i as f32 * CHARACTER_WIDTH)
                .collect::<Vec<_>>(),
        );
        node.set_character_widths(
            self.characters
                .iter()
                .map(|character| {
                    if is_hard_line_break(character) {
                        0.0
                    } else {
                        CHARACTER_WIDTH
                    }
                })
                .collect::<Vec<_>>(),
        );
        node.set_word_starts(self.word_starts.clone());
        if let Some(previous) = self.previous_on_line {
            node.set_previous_on_line(previous);
        }
        if let Some(next) = self.next_on_line {
            node.set_next_on_line(next);
        }
        node
    }
}
