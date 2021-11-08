use crate::atspi::{ObjectAddress, ObjectId, Role};
use std::convert::TryInto;
use zbus::names::OwnedUniqueName;
use zvariant::Str;

pub trait AccessibleInterface {
    fn name(&self) -> Str;

    fn description(&self) -> Str;

    fn parent(&self) -> Option<ObjectId>;

    fn child_count(&self) -> usize;

    fn locale(&self) -> Str;

    fn id(&self) -> ObjectId;

    fn child_at_index(&self, index: usize) -> Option<ObjectId>;

    fn children(&self) -> Vec<ObjectId>;

    fn index_in_parent(&self) -> Option<usize>;

    fn role(&self) -> Role;
}

pub struct AccessibleInterfaceObject<T> {
    bus_name: OwnedUniqueName,
    object: T,
}

#[dbus_interface(name = "org.a11y.atspi.Accessible")]
impl<T> AccessibleInterfaceObject<T>
where T: AccessibleInterface + Send + Sync + 'static
{
    #[dbus_interface(property)]
    fn name(&self) -> Str {
        self.object.name()
    }

    #[dbus_interface(property)]
    fn description(&self) -> Str {
        self.object.description()
    }

    #[dbus_interface(property)]
    fn parent(&self) -> ObjectAddress {
        if let Some(parent) = self.object.parent() {
            ObjectAddress::accessible(self.bus_name.as_ref(), parent)
        } else {
            ObjectAddress::null(self.bus_name.as_ref())
        }
    }

    #[dbus_interface(property)]
    fn child_count(&self) -> i32 {
        self.object.child_count().try_into().unwrap_or(0)
    }

    #[dbus_interface(property)]
    fn locale(&self) -> Str {
        self.object.locale()
    }

    #[dbus_interface(property)]
    fn accessible_id(&self) -> ObjectId {
        self.object.id()
    }

    fn get_child_at_index(&self, index: i32) -> ObjectAddress {
        index
        .try_into()
        .ok()
        .map(|index| self.object.child_at_index(index))
        .flatten()
        .map_or_else(|| ObjectAddress::null(self.bus_name.as_ref()), |id| ObjectAddress::accessible(self.bus_name.as_ref(), id))
    }

    fn get_children(&self) -> () {//&[ObjectAddress] {
        self.object.children();
    }

    fn get_index_in_parent(&self) -> i32 {
        self.object.index_in_parent().map_or(-1, |index| index.try_into().unwrap_or(-1))
    }

    fn get_role(&self) -> Role {
        self.object.role()
    }
}
