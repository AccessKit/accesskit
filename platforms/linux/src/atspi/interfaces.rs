use crate::atspi::{OwnedObjectAddress, Role};

pub trait ApplicationInterface {
    fn toolkit_name(&self) -> &str;

    fn toolkit_version(&self) -> &str;

    fn id(&self) -> i32;

    fn set_id(&mut self, id: i32);

    fn locale(&self, lctype: u32) -> &str;

    fn register_event_listener(&mut self, event: String);

    fn deregister_event_listener(&mut self, event: String);
}

pub struct ApplicationInterfaceObject<T>(T);

#[dbus_interface(name = "org.a11y.atspi.Application")]
impl<T: ApplicationInterface + Send + Sync + 'static> ApplicationInterfaceObject<T> {
    #[dbus_interface(property)]
    fn toolkit_name(&self) -> &str {
        self.0.toolkit_name()
    }

    #[dbus_interface(property)]
    fn version(&self) -> &str {
        self.0.toolkit_version()
    }

    #[dbus_interface(property)]
    fn atspi_version(&self) -> &str {
        "2.1"
    }

    #[dbus_interface(property)]
    fn id(&self) -> i32 {
        self.0.id()
    }

    #[dbus_interface(property)]
    fn set_id(&mut self, id: i32) {
        self.0.set_id(id)
    }

    fn get_locale(&self, lctype: u32) -> &str {
        self.0.locale(lctype)
    }

    fn register_event_listener(&self, _event: String) {}

    fn deregister_event_listener(&self, _event: String) {}
}

pub trait AccessibleInterface {
    fn name(&self) -> String;

    fn description(&self) -> String;

    fn parent(&self) -> Option<OwnedObjectAddress>;

    fn child_count(&self) -> usize;

    fn locale(&self) -> &str;

    fn accessible_id(&self) -> String;

    fn child_at_index(&self, index: usize) -> Option<OwnedObjectAddress>;

    fn children(&self) -> Vec<OwnedObjectAddress>;

    fn index_in_parent(&self) -> Option<usize>;

    fn role(&self) -> Role;
}

pub struct AccessibleInterfaceObject<T>(T);

#[dbus_interface(name = "org.a11y.atspi.Accessible")]
impl<T: AccessibleInterface + Send + Sync + 'static> AccessibleInterfaceObject<T> {
    #[dbus_interface(property)]
    fn name(&self) -> String {
        self.0.name()
    }

    #[dbus_interface(property)]
    fn description(&self) -> String {
        self.0.description()
    }

    #[dbus_interface(property)]
    fn parent(&self) -> OwnedObjectAddress {
        self.0.parent().unwrap()
    }

    #[dbus_interface(property)]
    fn child_count(&self) -> i32 {
        self.0.child_count() as i32
    }

    #[dbus_interface(property)]
    fn locale(&self) -> &str {
        ""
    }

    #[dbus_interface(property)]
    fn accessible_id(&self) -> String {
        self.0.accessible_id()
    }

    fn get_child_at_index(&self, index: i32) -> () {//ObjectAddress {
        self.0.child_at_index(index as usize).unwrap();
    }

    fn get_children(&self) -> () {//&[ObjectAddress] {
        self.0.children();
    }

    fn get_index_in_parent(&self) -> i32 {
        self.0.index_in_parent().unwrap() as i32
    }

    fn get_role(&self) -> Role {
        self.0.role()
    }
}
