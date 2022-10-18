// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

#![allow(non_upper_case_globals)]

use accesskit_consumer::{
    TextPosition as Position, TextRange as Range, Tree, TreeState, WeakTextRange as WeakRange,
};
use parking_lot::RwLock;
use std::sync::{Arc, Weak};
use windows::{
    core::*,
    Win32::{Foundation::*, System::Com::*, UI::Accessibility::*},
};

use crate::util::*;

fn upgrade_range<'a>(weak: &WeakRange, tree_state: &'a TreeState) -> Result<Range<'a>> {
    if let Some(range) = weak.upgrade(tree_state) {
        Ok(range)
    } else {
        Err(element_not_available())
    }
}

fn position_from_endpoint<'a>(
    range: &Range<'a>,
    endpoint: TextPatternRangeEndpoint,
) -> Result<Position<'a>> {
    match endpoint {
        TextPatternRangeEndpoint_Start => Ok(range.start()),
        TextPatternRangeEndpoint_End => Ok(range.end()),
        _ => Err(invalid_arg()),
    }
}

fn set_endpoint_position<'a>(range: &mut Range<'a>, endpoint: TextPatternRangeEndpoint, pos: Position<'a>) -> Result<()> {
    match endpoint {
        TextPatternRangeEndpoint_Start => {
            range.set_start(pos);
        }
        TextPatternRangeEndpoint_End => {
            range.set_end(pos);
        }
        _ => {
            return Err(invalid_arg());
        }
    }
    Ok(())
}

fn move_position_once<'a>(pos: Position<'a>, unit: TextUnit, forward: bool) -> Result<Option<Position<'a>>> {
    match unit {
        TextUnit_Character => {
            if forward {
                Ok(pos.forward_by_character())
            } else {
                Ok(pos.backward_by_character())
            }
        }
        TextUnit_Format => {
            if forward {
                Ok(pos.forward_by_format())
            } else {
                Ok(pos.backward_by_format())
            }
        }
        TextUnit_Word => {
            if forward {
                Ok(pos.forward_by_word())
            } else {
                Ok(pos.backward_by_word())
            }
        }
        TextUnit_Line => {
            if forward {
                Ok(pos.forward_by_line())
            } else {
                Ok(pos.backward_by_line())
            }
        }
        TextUnit_Paragraph => {
            if forward {
                Ok(pos.forward_by_paragraph())
            } else {
                Ok(pos.backward_by_paragraph())
            }
        }
        TextUnit_Page => {
            if forward {
                Ok(pos.forward_by_page())
            } else {
                Ok(pos.backward_by_page())
            }
        }
        TextUnit_Document => {
            if forward {
                Ok(pos.forward_by_document())
            } else {
                Ok(pos.backward_by_document())
            }
        }
        _ => Err(invalid_arg())
    }
}

fn move_position<'a>(mut pos: Position<'a>, unit: TextUnit, count: i32) -> Result<(Position<'a>, i32)> {
    let forward = count > 0;
    let count = count.abs();
    let mut moved = 0i32;
    for _ in 0..count {
        if let Some(new_pos) = move_position_once(pos, unit, forward)? {
            pos = new_pos;
            moved += 1;
        } else {
            break;
        }
    }
    if !forward {
        moved = -moved;
    }
    Ok((pos, moved))
}

#[implement(ITextRangeProvider)]
pub(crate) struct PlatformRange {
    tree: Weak<Tree>,
    state: RwLock<WeakRange>,
}

impl PlatformRange {
    pub(crate) fn new(tree: &Weak<Tree>, range: Range) -> Self {
        Self {
            tree: tree.clone(),
            state: RwLock::new(range.downgrade()),
        }
    }

    fn upgrade_tree(&self) -> Result<Arc<Tree>> {
        if let Some(tree) = self.tree.upgrade() {
            Ok(tree)
        } else {
            Err(element_not_available())
        }
    }

    fn with_tree_state<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&TreeState) -> Result<T>,
    {
        let tree = self.upgrade_tree()?;
        let state = tree.read();
        f(&state)
    }

    fn upgrade_for_read<'a>(&self, tree_state: &'a TreeState) -> Result<Range<'a>> {
        let state = self.state.read();
        upgrade_range(&state, tree_state)
    }

    fn read<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(Range) -> Result<T>,
    {
        self.with_tree_state(|tree_state| {
            let range = self.upgrade_for_read(tree_state)?;
            f(range)
        })
    }

    fn write<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Range) -> Result<T>,
    {
        self.with_tree_state(|tree_state| {
            let mut state = self.state.write();
            let mut range = upgrade_range(&state, tree_state)?;
            let result = f(&mut range);
            *state = range.downgrade();
            result
        })
    }

    fn require_same_tree(&self, other: &PlatformRange) -> Result<()> {
        if self.tree.ptr_eq(&other.tree) {
            Ok(())
        } else {
            Err(invalid_arg())
        }
    }
}

impl Clone for PlatformRange {
    fn clone(&self) -> Self {
        PlatformRange {
            tree: self.tree.clone(),
            state: RwLock::new(*self.state.read()),
        }
    }
}

// Some text range methods take another text range interface pointer as a
// parameter. We need to cast these interface pointers to their underlying
// implementations. We assume that AccessKit is the only UIA provider
// within this process. This seems a safe assumption for most AccessKit users.

impl ITextRangeProvider_Impl for PlatformRange {
    fn Clone(&self) -> Result<ITextRangeProvider> {
        Ok(self.clone().into())
    }

    fn Compare(&self, other: &Option<ITextRangeProvider>) -> Result<BOOL> {
        let other = required_param(other)?.as_impl();
        Ok((self.tree.ptr_eq(&other.tree) && *self.state.read() == *other.state.read()).into())
    }

    fn CompareEndpoints(
        &self,
        endpoint: TextPatternRangeEndpoint,
        other: &Option<ITextRangeProvider>,
        other_endpoint: TextPatternRangeEndpoint,
    ) -> Result<i32> {
        let other = required_param(other)?.as_impl();
        self.require_same_tree(&other)?;
        self.with_tree_state(|tree_state| {
            let range = self.upgrade_for_read(&tree_state)?;
            let other_range = other.upgrade_for_read(&tree_state)?;
            if range.node().id() != other_range.node().id() {
                return Err(invalid_arg());
            }
            let pos = position_from_endpoint(&range, endpoint)?;
            let other_pos = position_from_endpoint(&other_range, other_endpoint)?;
            let result = pos.partial_cmp(&other_pos).unwrap();
            Ok(result as i32)
        })
    }

    fn ExpandToEnclosingUnit(&self, unit: TextUnit) -> Result<()> {
        self.write(|range| {
            match unit {
                TextUnit_Character => {
                    range.expand_to_character();
                }
                TextUnit_Format => {
                    range.expand_to_format();
                }
                TextUnit_Word => {
                    range.expand_to_word();
                }
                TextUnit_Line => {
                    range.expand_to_line();
                }
                TextUnit_Paragraph => {
                    range.expand_to_paragraph();
                }
                TextUnit_Page => {
                    range.expand_to_page();
                }
                TextUnit_Document => {
                    range.expand_to_document();
                }
                _ => {
                    return Err(invalid_arg());
                }
            }
            Ok(())
        })
    }

    fn FindAttribute(
        &self,
        id: i32,
        value: &VARIANT,
        backward: BOOL,
    ) -> Result<ITextRangeProvider> {
        todo!()
    }

    fn FindText(
        &self,
        text: &BSTR,
        backward: BOOL,
        ignore_case: BOOL,
    ) -> Result<ITextRangeProvider> {
        todo!()
    }

    fn GetAttributeValue(&self, id: i32) -> Result<VARIANT> {
        todo!()
    }

    fn GetBoundingRectangles(&self) -> Result<*mut SAFEARRAY> {
        todo!()
    }

    fn GetEnclosingElement(&self) -> Result<IRawElementProviderSimple> {
        todo!()
    }

    fn GetText(&self, max_length: i32) -> Result<BSTR> {
        todo!()
    }

    fn Move(&self, unit: TextUnit, count: i32) -> Result<i32> {
        todo!()
    }

    fn MoveEndpointByUnit(
        &self,
        endpoint: TextPatternRangeEndpoint,
        unit: TextUnit,
        count: i32,
    ) -> Result<i32> {
        self.write(|range| {
            let pos = position_from_endpoint(range, endpoint)?;
            let (pos, moved) = move_position(pos, unit, count)?;
            set_endpoint_position(range, endpoint, pos)?;
            Ok(moved)
        })
    }

    fn MoveEndpointByRange(
        &self,
        endpoint: TextPatternRangeEndpoint,
        other: &Option<ITextRangeProvider>,
        other_endpoint: TextPatternRangeEndpoint,
    ) -> Result<()> {
        let other = required_param(other)?.as_impl();
        self.require_same_tree(&other)?;
        // We have to obtain the tree state and ranges manually to avoid
        // lifetime issues, and work with the two locks in a specific order
        // to avoid deadlock.
        self.with_tree_state(|tree_state| {
            let other_range = other.upgrade_for_read(tree_state)?;
            let mut state = self.state.write();
            let mut range = upgrade_range(&state, tree_state)?;
            if range.node().id() != other_range.node().id() {
                return Err(invalid_arg());
            }
            let pos = position_from_endpoint(&other_range, other_endpoint)?;
            set_endpoint_position(&mut range, endpoint, pos)?;
            *state = range.downgrade();
            Ok(())
        })
    }

    fn Select(&self) -> Result<()> {
        todo!()
    }

    fn AddToSelection(&self) -> Result<()> {
        todo!()
    }

    fn RemoveFromSelection(&self) -> Result<()> {
        todo!()
    }

    fn ScrollIntoView(&self, align_to_top: BOOL) -> Result<()> {
        todo!()
    }

    fn GetChildren(&self) -> Result<*mut SAFEARRAY> {
        // We don't support embedded objects in text.
        Ok(std::ptr::null_mut())
    }
}