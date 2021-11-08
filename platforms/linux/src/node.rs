use accesskit_consumer::{Node, WeakNode};
use crate::atspi::{
    interfaces::{AccessibleInterface, ApplicationInterface},
    ObjectId, Role
};
use zvariant::Str;

pub struct PlatformNode(WeakNode);

impl PlatformNode {
    pub(crate) fn new(node: &Node) -> Self {
        Self(node.downgrade())
    }
}

impl AccessibleInterface for PlatformNode {
    fn name(&self) -> Str {
        self.0.map(|node| {
            match node.name() {
                None => Str::default(),
                Some(name) => Str::from(name.to_string())
            }
        }).unwrap()
    }

    fn description(&self) -> Str {
        todo!()
    }

    fn parent(&self) -> Option<ObjectId> {
        todo!()
    }

    fn child_count(&self) -> usize {
        todo!()
    }

    fn locale(&self) -> Str {
        todo!()
    }

    fn id(&self) -> ObjectId {
        todo!()
    }

    fn child_at_index(&self, index: usize) -> Option<ObjectId> {
        todo!()
    }

    fn children(&self) -> Vec<ObjectId> {
        todo!()
    }

    fn index_in_parent(&self) -> Option<usize> {
        todo!()
    }

    fn role(&self) -> Role {
        todo!()
    }
}

pub struct RootPlatformNode {
    app_name: String,
    app_id: i32,
    toolkit_name: String,
    toolkit_version: String,
}

impl RootPlatformNode {
    pub fn new(app_name: String, toolkit_name: String, toolkit_version: String) -> Self {
        Self {
            app_name,
            app_id: -1,
            toolkit_name,
            toolkit_version
        }
    }
}

impl ApplicationInterface for RootPlatformNode {
    fn name(&self) -> Str {
        Str::from(&self.app_name)
    }

    fn children(&self) -> Vec<ObjectId> {
        Vec::new()
    }

    fn toolkit_name(&self) -> Str {
        Str::from(&self.toolkit_name)
    }

    fn toolkit_version(&self) -> Str {
        Str::from(&self.toolkit_version)
    }

    fn id(&self) -> i32 {
        self.app_id
    }

    fn set_id(&mut self, id: i32) {
        self.app_id = id;
    }

    fn locale(&self, lctype: u32) -> Str {
        Str::default()
    }

    fn register_event_listener(&mut self, event: String) {
    }

    fn deregister_event_listener(&mut self, event: String) {
    }
}
