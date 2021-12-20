// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use enumflags2::{bitflags, BitFlag, BitFlags, FromBitsError};
use serde::{
    de::{self, Deserialize, Deserializer, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};
use std::fmt;
use strum::AsRefStr;
use zvariant::{Signature, Type};

/// Enumeration used by various interfaces indicating every possible state
/// an #AtspiAccessible object can assume.
#[bitflags]
#[repr(u64)]
#[derive(AsRefStr, Clone, Copy, Debug)]
#[strum(serialize_all = "kebab-case")]
pub enum State {
    /// Indicates an invalid state - probably an error condition.
    Invalid,
    /// Indicates a window is currently the active window, or
    /// an object is the active subelement within a container or table.
    /// @ACTIVE should not be used for objects which have
    /// #FOCUSABLE or #SELECTABLE: Those objects should use
    /// @FOCUSED and @SELECTED respectively.
    /// @ACTIVE is a means to indicate that an object which is not
    /// focusable and not selectable is the currently-active item within its
    /// parent container.
    Active,
    /// Indicates that the object is armed.
    Armed,
    /// Indicates the current object is busy, i.e. onscreen
    /// representation is in the process of changing, or       the object is
    /// temporarily unavailable for interaction due to activity already in progress.
    Busy,
    /// Indicates this object is currently checked.
    Checked,
    /// Indicates this object is collapsed.
    Collapsed,
    /// Indicates that this object no longer has a valid
    /// backing widget        (for instance, if its peer object has been destroyed).
    Defunct,
    /// Indicates the user can change the contents of this object.
    Editable,
    /// Indicates that this object is enabled, i.e. that it
    /// currently reflects some application state. Objects that are "greyed out"
    /// may lack this state, and may lack the @SENSITIVE if direct
    /// user interaction cannot cause them to acquire @ENABLED.
    /// See @SENSITIVE.
    Enabled,
    /// Indicates this object allows progressive
    /// disclosure of its children.
    Expandable,
    /// Indicates this object is expanded.
    Expanded,
    /// Indicates this object can accept keyboard focus,
    /// which means all events resulting from typing on the keyboard will
    /// normally be passed to it when it has focus.
    Focusable,
    /// Indicates this object currently has the keyboard focus.
    Focused,
    /// Indicates that the object has an associated tooltip.
    HasTooltip,
    /// Indicates the orientation of this object is horizontal.
    Horizontal,
    /// Indicates this object is minimized and is
    /// represented only by an icon.
    Iconified,
    /// Indicates something must be done with this object
    /// before the user can interact with an object in a different window.
    Modal,
    /// Indicates this (text) object can contain multiple
    /// lines of text.
    MultiLine,
    /// Indicates this object allows more than one of
    /// its children to be selected at the same time, or in the case of text
    /// objects, that the object supports non-contiguous text selections.
    Multiselectable,
    /// Indicates this object paints every pixel within its
    /// rectangular region. It also indicates an alpha value of unity, if it
    /// supports alpha blending.
    Opaque,
    /// Indicates this object is currently pressed.
    Pressed,
    /// Indicates the size of this object's size is not fixed.
    Resizable,
    /// Indicates this object is the child of an object
    /// that allows its children to be selected and that this child is one of
    /// those children       that can be selected.
    Selectable,
    /// Indicates this object is the child of an object that
    /// allows its children to be selected and that this child is one of those
    /// children that has been selected.
    Selected,
    /// Indicates this object is sensitive, e.g. to user
    /// interaction. @SENSITIVE usually accompanies.
    /// @ENABLED for user-actionable controls, but may be found in the
    /// absence of @ENABLED if the current visible state of the control
    /// is "disconnected" from the application state.  In such cases, direct user
    /// interaction can often result in the object gaining @SENSITIVE,
    /// for instance if a user makes an explicit selection using an object whose
    /// current state is ambiguous or undefined. See @ENABLED,
    /// @INDETERMINATE.
    Sensitive,
    /// Indicates this object, the object's parent, the
    /// object's parent's parent, and so on, are all 'shown' to the end-user,
    /// i.e. subject to "exposure" if blocking or obscuring objects do not
    /// interpose between this object and the top of the window stack.
    Showing,
    /// Indicates this (text) object can contain only a
    /// single line of text.
    SingleLine,
    /// Indicates that the information returned for this object
    /// may no longer be synchronized with the application state.  This can occur
    /// if the object has @TRANSIENT, and can also occur towards the
    /// end of the object peer's lifecycle.
    Stale,
    /// Indicates this object is transient.
    Transient,
    /// Indicates the orientation of this object is vertical;
    /// for example this state may appear on such objects as scrollbars, text
    /// objects (with vertical text flow), separators, etc.
    Vertical,
    /// Indicates this object is visible, e.g. has been
    /// explicitly marked for exposure to the user. @VISIBLE is no
    /// guarantee that the object is actually unobscured on the screen, only that
    /// it is 'potentially' visible, barring obstruction, being scrolled or clipped
    /// out of the field of view, or having an ancestor container that has not yet
    /// made visible. A widget is potentially onscreen if it has both
    /// @VISIBLE and @SHOWING. The absence of
    /// @VISIBLE and @SHOWING is
    /// semantically equivalent to saying that an object is 'hidden'.
    Visible,
    /// Indicates that "active-descendant-changed"
    /// event is sent when children become 'active' (i.e. are selected or
    /// navigated to onscreen).  Used to prevent need to enumerate all children
    /// in very large containers, like tables. The presence of
    /// @MANAGES_DESCENDANTS is an indication to the client that the
    /// children should not, and need not, be enumerated by the client.
    /// Objects implementing this state are expected to provide relevant state      
    /// notifications to listening clients, for instance notifications of
    /// visibility changes and activation of their contained child objects, without
    /// the client having previously requested references to those children.
    ManagesDescendants,
    /// Indicates that a check box or other boolean
    /// indicator is in a state other than checked or not checked.  This
    /// usually means that the boolean value reflected or controlled by the
    /// object does not apply consistently to the entire current context.      
    /// For example, a checkbox for the "Bold" attribute of text may have
    /// @INDETERMINATE if the currently selected text contains a mixture
    /// of weight attributes. In many cases interacting with a
    /// @INDETERMINATE object will cause the context's corresponding
    /// boolean attribute to be homogenized, whereupon the object will lose
    /// @INDETERMINATE and a corresponding state-changed event will be
    /// fired.
    Indeterminate,
    /// Indicates that user interaction with this object is
    /// 'required' from the user, for instance before completing the
    /// processing of a form.
    Required,
    /// Indicates that an object's onscreen content
    /// is truncated, e.g. a text value in a spreadsheet cell.
    Truncated,
    /// Indicates this object's visual representation is
    /// dynamic, not static. This state may be applied to an object during an
    /// animated 'effect' and be removed from the object once its visual
    /// representation becomes static. Some applications, notably content viewers,
    /// may not be able to detect all kinds of animated content.  Therefore the
    /// absence of this state should not be taken as
    /// definitive evidence that the object's visual representation is      
    /// static; this state is advisory.
    Animated,
    /// This object has indicated an error condition
    /// due to failure of input validation.  For instance, a form control may
    /// acquire this state in response to invalid or malformed user input.
    InvalidEntry,
    /// This state indicates that the object
    /// in question implements some form of typeahead or       
    /// pre-selection behavior whereby entering the first character of one or more
    /// sub-elements causes those elements to scroll into view or become
    /// selected. Subsequent character input may narrow the selection further as
    /// long as one or more sub-elements match the string. This state is normally
    /// only useful and encountered on objects that implement #AtspiSelection.
    /// In some cases the typeahead behavior may result in full or partial
    /// completion of the data in the input field, in which case
    /// these input events may trigger text-changed events from the source.
    SupportsAutocompletion,
    /// This state indicates that the object in
    /// question supports text selection. It should only be exposed on objects
    /// which implement the #AtspiText interface, in order to distinguish this state
    /// from @SELECTABLE, which infers that the object in question is a
    /// selectable child of an object which implements #AtspiSelection. While
    /// similar, text selection and subelement selection are distinct operations.
    SelectableText,
    /// This state indicates that the object in question is
    /// the 'default' interaction object in a dialog, i.e. the one that gets
    /// activated if the user presses "Enter" when the dialog is initially
    /// posted.
    IsDefault,
    /// This state indicates that the object (typically a
    /// hyperlink) has already been activated or invoked, with the result that
    /// some backing data has been downloaded or rendered.
    Visited,
    /// Indicates this object has the potential to
    ///  be checked, such as a checkbox or toggle-able table cell. @Since:
    /// 2.12
    Checkable,
    /// Indicates that the object has a popup
    /// context menu or sub-level menu which may or may not be
    /// showing. This means that activation renders conditional content.
    /// Note that ordinary tooltips are not considered popups in this
    /// context. @Since: 2.12
    HasPopup,
    /// Indicates that an object which is ENABLED and
    /// SENSITIVE has a value which can be read, but not modified, by the
    /// user. @Since: 2.16
    ReadOnly,
    /// This value of the enumeration should not be used
    /// as a parameter, it indicates the number of items in the #AtspiStateType
    /// enumeration.
    LastDefined,
}

#[derive(Clone, Copy, Debug)]
pub struct StateSet(BitFlags<State>);

impl StateSet {
    pub fn from_bits(bits: u64) -> Result<StateSet, FromBitsError<State>> {
        Ok(StateSet(BitFlags::from_bits(bits)?))
    }

    pub fn empty() -> StateSet {
        StateSet(State::empty())
    }

    pub fn bits(&self) -> u64 {
        self.0.bits()
    }

    pub fn contains<B: Into<BitFlags<State>>>(self, other: B) -> bool {
        self.0.contains(other)
    }

    pub fn insert<B: Into<BitFlags<State>>>(&mut self, other: B) {
        self.0.insert(other);
    }

    pub fn iter(self) -> impl Iterator<Item = State> {
        self.0.iter()
    }
}

impl<'de> Deserialize<'de> for StateSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StateSetVisitor;

        impl<'de> Visitor<'de> for StateSetVisitor {
            type Value = StateSet;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter
                    .write_str("a sequence comprised of two u32 that represents a valid StateSet")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                match SeqAccess::next_element::<Vec<u32>>(&mut seq)? {
                    Some(vec) => {
                        let len = vec.len();
                        if len != 2 {
                            return Err(de::Error::invalid_length(len, &"Vec with two elements"));
                        }
                        Ok(StateSet::from_bits(0).unwrap())
                    }
                    None => Err(de::Error::custom("Vec with two elements")),
                }
            }
        }

        deserializer.deserialize_seq(StateSetVisitor)
    }
}

impl Serialize for StateSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(2))?;
        let bits = self.bits();
        seq.serialize_element(&(bits as u32))?;
        seq.serialize_element(&((bits >> 32) as u32))?;
        seq.end()
    }
}

impl Type for StateSet {
    fn signature() -> Signature<'static> {
        Signature::from_str_unchecked("au")
    }
}

impl From<State> for StateSet {
    fn from(value: State) -> Self {
        Self(value.into())
    }
}

impl std::ops::BitXor for StateSet {
    type Output = StateSet;

    fn bitxor(self, other: Self) -> Self::Output {
        StateSet(self.0 ^ other.0)
    }
}
