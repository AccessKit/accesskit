use crate::atspi::{
    interfaces::AccessibleInterface,
    ObjectId, Role
};
use zvariant::Str;

pub trait ApplicationInterface {
    fn name(&self) -> Str;

    fn children(&self) -> Vec<ObjectId>;

    fn toolkit_name(&self) -> Str;

    fn toolkit_version(&self) -> Str;

    fn id(&self) -> i32;

    fn set_id(&mut self, id: i32);

    fn locale(&self, lctype: u32) -> Str;

    fn register_event_listener(&mut self, event: String);

    fn deregister_event_listener(&mut self, event: String);
}

impl<T> AccessibleInterface for T
where T: ApplicationInterface
{
    fn name(&self) -> Str {
        self.name()
    }

    fn description(&self) -> Str {
        Str::default()
    }

    fn parent(&self) -> Option<ObjectId> {
        todo!()
    }

    fn child_count(&self) -> usize {
        todo!()
    }

    fn locale(&self) -> Str {
        Str::default()
    }

    fn id(&self) -> ObjectId {
        unsafe {
            ObjectId::from_str_unchecked("root")
        }
    }

    fn child_at_index(&self, index: usize) -> Option<ObjectId> {
        todo!()
    }

    fn children(&self) -> Vec<ObjectId> {
        todo!()
    }

    fn index_in_parent(&self) -> Option<usize> {
        None
    }

    fn role(&self) -> Role {
        Role::Application
    }
}

pub struct ApplicationInterfaceObject<T>(pub T);

#[dbus_interface(name = "org.a11y.atspi.Application")]
impl<T> ApplicationInterfaceObject<T>
where T: ApplicationInterface + Send + Sync + 'static
{
    #[dbus_interface(property)]
    fn toolkit_name(&self) -> Str {
        self.0.toolkit_name()
    }

    #[dbus_interface(property)]
    fn version(&self) -> Str {
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

    fn get_locale(&self, lctype: u32) -> Str {
        self.0.locale(lctype)
    }

    fn register_event_listener(&self, _event: String) {}

    fn deregister_event_listener(&self, _event: String) {}
}
