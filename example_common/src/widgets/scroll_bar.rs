use accesskit::{Action, ActionData, ActionRequest, Node, NodeId, Orientation, Rect, Role};

use super::{Response, Widget};

/// How far a container is scrolled.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollPosition {
    /// A distance from the start.
    Offset(f64),
    /// The end, even as the content grows.
    End,
}

impl ScrollPosition {
    /// The position at `offset`, when the container scrolls up to `max`.
    pub(super) fn at(offset: f64, max: f64) -> Self {
        if offset >= max {
            Self::End
        } else {
            Self::Offset(offset.max(0.0))
        }
    }

    /// The offset of the position, when the container scrolls up to `max`.
    pub(super) fn offset(self, max: f64) -> f64 {
        match self {
            Self::Offset(offset) => offset.clamp(0.0, max),
            Self::End => max,
        }
    }
}

/// A scrollbar for a container, vertical unless set otherwise.
///
/// It can't be focused. Its numeric value is the scroll position of the
/// container, which it marks as the node it controls.
pub struct ScrollBar<M> {
    id: NodeId,
    bounds: Rect,
    orientation: Orientation,
    controls: NodeId,
    offset: f64,
    max: f64,
    step: Option<f64>,
    jump: Option<f64>,
    on_scroll: fn(ScrollPosition) -> M,
}

impl<M> ScrollBar<M> {
    /// Creates a scrollbar for the container `controls`, which scrolls up to
    /// `max`. `on_scroll` receives the new position.
    pub fn new(
        id: NodeId,
        bounds: Rect,
        controls: NodeId,
        position: ScrollPosition,
        max: f64,
        on_scroll: fn(ScrollPosition) -> M,
    ) -> Self {
        Self {
            id,
            bounds,
            orientation: Orientation::Vertical,
            controls,
            offset: position.offset(max),
            max,
            step: None,
            jump: None,
            on_scroll,
        }
    }

    /// Sets the direction the scrollbar scrolls in.
    pub fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Sets how far the scrollbar scrolls when incremented or decremented,
    /// which it can't be otherwise.
    pub fn with_step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }

    /// Sets how far a page is, as when clicking the scrollbar's track.
    pub fn with_jump(mut self, jump: f64) -> Self {
        self.jump = Some(jump);
        self
    }

    fn scroll_to(&self, offset: f64) -> Response<M> {
        let offset = offset.clamp(0.0, self.max);
        if offset == self.offset {
            Response::Handled
        } else {
            Response::Message((self.on_scroll)(ScrollPosition::at(offset, self.max)))
        }
    }
}

impl<M> Widget<M> for ScrollBar<M> {
    fn id(&self) -> NodeId {
        self.id
    }

    fn build(&self) -> Node {
        let mut node = Node::new(Role::ScrollBar);
        node.set_bounds(self.bounds);
        node.set_label(match self.orientation {
            Orientation::Horizontal => "Horizontal scrollbar",
            Orientation::Vertical => "Vertical scrollbar",
        });
        node.set_orientation(self.orientation);
        node.set_controls(vec![self.controls]);
        node.set_numeric_value(self.offset);
        node.set_min_numeric_value(0.0);
        node.set_max_numeric_value(self.max);
        node.add_action(Action::SetValue);
        if let Some(step) = self.step {
            node.set_numeric_value_step(step);
            node.add_action(Action::Increment);
            node.add_action(Action::Decrement);
        }
        if let Some(jump) = self.jump {
            node.set_numeric_value_jump(jump);
        }
        node
    }

    fn on_action(&self, request: &ActionRequest) -> Response<M> {
        match (request.action, &request.data) {
            (Action::SetValue, Some(ActionData::NumericValue(value))) => self.scroll_to(*value),
            (Action::Increment, _) => self
                .step
                .map_or(Response::Ignored, |step| self.scroll_to(self.offset + step)),
            (Action::Decrement, _) => self
                .step
                .map_or(Response::Ignored, |step| self.scroll_to(self.offset - step)),
            _ => Response::Ignored,
        }
    }
}
