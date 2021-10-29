use accesskit_schema::{Node, NodeId, Role, StringEncoding, Tree, TreeId, TreeUpdate};
use accesskit_linux::Manager;
use std::num::NonZeroU64;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const WINDOW_TITLE: &str = "Hello world";

const WINDOW_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(1) });
const BUTTON_1_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(2) });
const BUTTON_2_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(3) });

fn get_tree(is_window_focused: bool) -> Tree {
    Tree {
        focus: is_window_focused.then(|| unsafe { FOCUS }),
        ..Tree::new(TreeId("test".into()), StringEncoding::Utf8)
    }
}

fn get_button_1(name: &str) -> Node {
    Node {
        name: Some(name.into()),
        focusable: true,
        ..Node::new(BUTTON_1_ID, Role::Button)
    }
}

fn get_button_2(name: &str) -> Node {
    Node {
        name: Some(name.into()),
        focusable: true,
        ..Node::new(BUTTON_2_ID, Role::Button)
    }
}

fn get_initial_state() -> TreeUpdate {
    let root = Node {
        children: Box::new([BUTTON_1_ID, BUTTON_2_ID]),
        name: Some(WINDOW_TITLE.into()),
        ..Node::new(WINDOW_ID, Role::Window)
    };
    let button_1 = get_button_1("Button 1");
    let button_2 = get_button_2("Button 2");
    TreeUpdate {
        clear: None,
        nodes: vec![root, button_1, button_2],
        tree: Some(get_tree(false)),
        root: Some(WINDOW_ID),
    }
}

static mut FOCUS: NodeId = BUTTON_1_ID;

fn main() {
    let manager = Manager::new(get_initial_state());
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .with_inner_size(winit::dpi::LogicalSize::new(128.0, 128.0))
        .build(&event_loop)
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => (),
        }
    });
}
