/// This is a simple example for creating a headless
/// application using accesskit_unix. It will not create any GUI
/// windows on the system. Rather it just creates the necessary
/// accessibility tree and updates it in a loop with a live announcement.
/// This will be spoken by Orca and works on both x11 and Wayland.
///
/// To run this example:
/// cargo run -p accesskit_unix --example simple_headless
use accesskit::{
    ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, Live, Node, NodeId, Role,
    Tree, TreeUpdate,
};
use accesskit_unix::Adapter;

use std::error::Error;
use std::thread;
use std::time::Duration;

const ROOT_ID: NodeId = NodeId(0);
const ANNOUNCE_ID: NodeId = NodeId(1);

// These three structs are defined in order
// to fulfill the requirements for creating an
// adapter and initializing accesskit
struct MyActivationHandler;
struct MyActionHandler;
struct MyDeactivationHandler;

impl ActivationHandler for MyActivationHandler {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        let mut root = Node::new(Role::Window);
        root.set_children(vec![ANNOUNCE_ID]);
        // The label on the window is equivalent to the frame
        // title that Orca reads
        root.set_label("Window for Headless Example");

        let mut label = Node::new(Role::Label);
        label.set_value("Application started");
        label.set_live(Live::Assertive);

        let tree = Tree::new(ROOT_ID);
        Some(TreeUpdate {
            // A node with id ANNOUNCE_ID must be in the nodes otherwise you will get an error
            // This is since we are later updating it with a live announcement
            nodes: vec![(ROOT_ID, root), (ANNOUNCE_ID, label)],
            tree: Some(tree),
            focus: ROOT_ID,
        })
    }
}

// In a full application you would implement the behavior of the actions
// within the GUI. We don't need to do anything here since we run
// the tree update in a loop.
impl ActionHandler for MyActionHandler {
    fn do_action(&mut self, request: ActionRequest) {
        println!("Got Action: {:?}", request);
    }
}

// The deactivation handler runs a cleanup function before accessibility is deactivated
// In our case this can be skipped
impl DeactivationHandler for MyDeactivationHandler {
    fn deactivate_accessibility(&mut self) {
        println!("Accessibility deactivated");
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("This is a simple headless example of accesskit_unix.");
    println!("This will not show any GUI windows on the system, however it will create a live announcement in a loop.");
    println!("Orca should speak these announcements.");
    println!("Enable Orca with [Super]+[Alt]+[S].");

    // Create adapter and activate accessibility
    let mut adapter = Adapter::new(MyActivationHandler, MyActionHandler, MyDeactivationHandler);
    // At the start of the application we give the window focus;
    // this is not needed for Orca to speak an announcement and
    // could be skipped if you don't want to change the focus
    adapter.update_window_focus_state(true);

    adapter.update_if_active(|| MyActivationHandler.request_initial_tree().unwrap());

    let mut counter = 0;
    loop {
        counter += 1;
        let message = format!("Announcement number {}", counter);
        // Print to stdout for clarity and debugging
        println!("Creating tree update with announcement: '{message}'");

        let mut label = Node::new(Role::Label);
        label.set_value(message);
        label.set_live(Live::Polite);

        // when we update the tree with an announcement
        // it can be detected by the assistive technology client
        adapter.update_if_active(|| TreeUpdate {
            nodes: vec![(ANNOUNCE_ID, label)],
            tree: None,
            focus: ROOT_ID,
        });

        // sleep for 1 second so as to not spam the user
        thread::sleep(Duration::from_secs(1));
    }
}
