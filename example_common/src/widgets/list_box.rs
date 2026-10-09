use accesskit::{
    Action, ActionData, ActionRequest, Live, Node, NodeId, Orientation, Rect, Role, ScrollHint,
    ScrollUnit, Vec2,
};

use super::{Response, ScrollBar, ScrollPosition, Widget};
use crate::{Key, KeyEvent, KeyState};

/// An item of a [`ListBox`].
pub struct ListItem {
    pub id: NodeId,
    pub label: String,
    /// Where the item is when the list isn't scrolled.
    pub bounds: Rect,
    /// Whether screen readers announce the item when it's added, and how.
    pub live: Option<Live>,
}

struct ListBoxOption {
    id: NodeId,
    label: String,
    bounds: Rect,
    live: Option<Live>,
    is_selected: bool,
    position_in_set: Option<usize>,
}

impl<M> Widget<M> for ListBoxOption {
    fn id(&self) -> NodeId {
        self.id
    }

    fn build(&self) -> Node {
        let mut node = Node::new(Role::ListBoxOption);
        node.set_bounds(self.bounds);
        node.set_label(self.label.as_str());
        node.set_selected(self.is_selected);
        if let Some(live) = self.live {
            node.set_live(live);
        }
        if let Some(position) = self.position_in_set {
            node.set_position_in_set(position);
        }
        node.add_action(Action::Focus);
        node.add_action(Action::Click);
        node
    }
}

struct Scrolling<M> {
    offset: f64,
    max: f64,
    step: f64,
    padding: f64,
    on_scroll: fn(ScrollPosition) -> M,
}

/// A list of items, one of which may be selected.
///
/// The list keeps keyboard focus, and its selected item is its active
/// descendant.
pub struct ListBox<M> {
    id: NodeId,
    label: String,
    bounds: Rect,
    options: Vec<ListBoxOption>,
    selected: Option<usize>,
    has_set_info: bool,
    on_select: fn(usize) -> M,
    scrolling: Option<Scrolling<M>>,
}

impl<M> ListBox<M> {
    /// Creates a list box. `on_select` receives the index of the item to
    /// select.
    pub fn new(
        id: NodeId,
        label: impl Into<String>,
        bounds: Rect,
        items: Vec<ListItem>,
        selected: Option<usize>,
        on_select: fn(usize) -> M,
    ) -> Self {
        let options = items
            .into_iter()
            .enumerate()
            .map(|(index, item)| ListBoxOption {
                id: item.id,
                label: item.label,
                bounds: item.bounds,
                live: item.live,
                is_selected: Some(index) == selected,
                position_in_set: None,
            })
            .collect();
        Self {
            id,
            label: label.into(),
            bounds,
            options,
            selected,
            has_set_info: false,
            on_select,
            scrolling: None,
        }
    }

    /// Sets whether the list exposes its size and the position of its items.
    pub fn with_set_info(mut self, has_set_info: bool) -> Self {
        self.has_set_info = has_set_info;
        for (index, option) in self.options.iter_mut().enumerate() {
            option.position_in_set = has_set_info.then_some(index);
        }
        self
    }

    /// Makes the list scroll vertically, clipping its items. `on_scroll`
    /// receives the new position.
    pub fn with_scrolling(
        mut self,
        position: ScrollPosition,
        on_scroll: fn(ScrollPosition) -> M,
    ) -> Self {
        let padding = self
            .options
            .first()
            .map_or(0.0, |first| (first.bounds.y0 - self.bounds.y0).max(0.0));
        let content_bottom = self
            .options
            .last()
            .map_or(self.bounds.y0, |last| last.bounds.y1 + padding);
        let max = (content_bottom - self.bounds.y1).max(0.0);
        let step = match self.options.as_slice() {
            [first, second, ..] => second.bounds.y0 - first.bounds.y0,
            [only] => only.bounds.height(),
            [] => 0.0,
        };
        let offset = position.offset(max);
        for option in &mut self.options {
            option.bounds = option.bounds - Vec2::new(0.0, offset);
        }
        self.scrolling = Some(Scrolling {
            offset,
            max,
            step,
            padding,
            on_scroll,
        });
        self
    }

    /// Creates a scrollbar for the list, if it has anything to scroll.
    pub fn vertical_scroll_bar(&self, id: NodeId, bounds: Rect) -> Option<ScrollBar<M>> {
        let scrolling = self
            .scrolling
            .as_ref()
            .filter(|scrolling| scrolling.max > 0.0)?;
        Some(
            ScrollBar::new(
                id,
                bounds,
                self.id,
                ScrollPosition::at(scrolling.offset, scrolling.max),
                scrolling.max,
                scrolling.on_scroll,
            )
            .with_step(scrolling.step)
            .with_jump(self.bounds.height()),
        )
    }

    fn scroll_to(&self, offset: f64) -> Option<M> {
        let scrolling = self.scrolling.as_ref()?;
        let offset = offset.clamp(0.0, scrolling.max);
        (offset != scrolling.offset)
            .then(|| (scrolling.on_scroll)(ScrollPosition::at(offset, scrolling.max)))
    }

    fn scroll_into_view(&self, index: usize, hint: Option<ScrollHint>) -> Option<M> {
        let scrolling = self.scrolling.as_ref()?;
        let bounds = self.options[index].bounds;
        let above = bounds.y0 - (self.bounds.y0 + scrolling.padding);
        let below = bounds.y1 - (self.bounds.y1 - scrolling.padding);
        let delta = match hint {
            Some(ScrollHint::TopLeft | ScrollHint::TopEdge) => above,
            Some(ScrollHint::BottomRight | ScrollHint::BottomEdge) => below,
            _ if above < 0.0 => above,
            _ if below > 0.0 => below.min(above),
            _ => 0.0,
        };
        self.scroll_to(scrolling.offset + delta)
    }

    /// Scrolls by the given unit, or by a page when there is none, as when
    /// VoiceOver scrolls on iOS.
    fn scroll_by_unit(&self, action: Action, unit: Option<ScrollUnit>) -> Response<M> {
        let Some(scrolling) = &self.scrolling else {
            return Response::Ignored;
        };
        let distance = match unit {
            Some(ScrollUnit::Item) => scrolling.step,
            Some(ScrollUnit::Page) | None => self.bounds.height(),
        };
        let offset = match action {
            Action::ScrollUp => scrolling.offset - distance,
            _ => scrolling.offset + distance,
        };
        self.scroll_to(offset)
            .map_or(Response::Handled, Response::Message)
    }
}

impl<M> Widget<M> for ListBox<M> {
    fn id(&self) -> NodeId {
        self.id
    }

    fn build(&self) -> Node {
        let mut node = Node::new(Role::ListBox);
        node.set_bounds(self.bounds);
        node.set_label(self.label.as_str());
        node.set_orientation(Orientation::Vertical);
        node.set_children(
            self.options
                .iter()
                .map(|option| option.id)
                .collect::<Vec<_>>(),
        );
        if let Some(index) = self.selected {
            node.set_active_descendant(self.options[index].id);
        }
        if self.has_set_info {
            node.set_size_of_set(self.options.len());
        }
        node.add_action(Action::Focus);
        if let Some(scrolling) = &self.scrolling {
            node.set_clips_children();
            node.set_scroll_y(scrolling.offset);
            node.set_scroll_y_min(0.0);
            node.set_scroll_y_max(scrolling.max);
            if scrolling.offset > 0.0 {
                node.add_action(Action::ScrollUp);
            }
            if scrolling.offset < scrolling.max {
                node.add_action(Action::ScrollDown);
            }
            node.add_child_action(Action::ScrollIntoView);
        }
        node
    }

    fn children(&self) -> Vec<&dyn Widget<M>> {
        self.options.iter().map(|option| option as _).collect()
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn on_key(&self, event: &KeyEvent, _focus: NodeId) -> Response<M> {
        let Some(last) = self.options.len().checked_sub(1) else {
            return Response::Ignored;
        };
        if event.state != KeyState::Pressed {
            return Response::Ignored;
        }
        let index = match (event.key, self.selected) {
            (Key::Up, Some(index)) => index.saturating_sub(1),
            (Key::Down, Some(index)) => (index + 1).min(last),
            (Key::Up | Key::End, _) => last,
            (Key::Down | Key::Home, _) => 0,
            _ => return Response::Ignored,
        };
        let select = (Some(index) != self.selected).then(|| (self.on_select)(index));
        let scroll = self.scroll_into_view(index, None);
        Response::Messages(select.into_iter().chain(scroll).collect())
    }

    fn on_action(&self, request: &ActionRequest) -> Response<M> {
        if request.target_node == self.id {
            return match (request.action, &request.data) {
                (Action::Focus, _) => Response::Focus(self.id),
                (Action::ScrollUp | Action::ScrollDown, data) => {
                    let unit = match data {
                        Some(ActionData::ScrollUnit(unit)) => Some(*unit),
                        _ => None,
                    };
                    self.scroll_by_unit(request.action, unit)
                }
                _ => Response::Ignored,
            };
        }
        let Some(index) = self
            .options
            .iter()
            .position(|option| option.id == request.target_node)
        else {
            return Response::Ignored;
        };
        match (request.action, &request.data) {
            (Action::Focus | Action::Click, _) if Some(index) == self.selected => {
                Response::Focus(self.id)
            }
            (Action::Focus | Action::Click, _) => {
                Response::FocusAndMessage(self.id, (self.on_select)(index))
            }
            (Action::ScrollIntoView, data) => {
                let hint = match data {
                    Some(ActionData::ScrollHint(hint)) => Some(*hint),
                    _ => None,
                };
                self.scroll_into_view(index, hint)
                    .map_or(Response::Handled, Response::Message)
            }
            _ => Response::Ignored,
        }
    }
}
