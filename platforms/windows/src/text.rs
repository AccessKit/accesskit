// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

#![allow(non_upper_case_globals)]

use accesskit_consumer::{TextPosition as Position, TextRange as Range, Tree, TreeState, WeakTextRange as WeakRange};
use parking_lot::RwLock;
use std::sync::{Arc, Weak};
use windows::{
    core::*,
    Win32::{Foundation::*, System::Com::*, UI::Accessibility::*},
};

use crate::util::*;

fn position_from_endpoint<'a>(range: &'a Range, endpoint: TextPatternRangeEndpoint) -> Result<Position<'a>> {
    match endpoint {
        TextPatternRangeEndpoint_Start => Ok(range.start()),
        TextPatternRangeEndpoint_End => Ok(range.end()),
        _ => Err(invalid_arg()),
    }
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

    fn read<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Range) -> Result<T>,
    {
        self.with_tree_state(|tree_state| {
            let state = self.state.read();
            if let Some(range) = state.upgrade(tree_state) {
                f(&range)
            } else {
                Err(element_not_available())
            }
        })
    }

    fn write<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Range) -> Result<T>,
    {
        self.with_tree_state(|tree_state| {
            let mut state = self.state.write();
            if let Some(mut range) = state.upgrade(tree_state) {
                let result = f(&mut range);
                *state = range.downgrade();
                result
            } else {
                Err(element_not_available())
            }
        })
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
        Ok((*self.state.read() == *other.state.read()).into())
    }

    fn CompareEndpoints(
        &self,
        endpoint: TextPatternRangeEndpoint,
        other: &Option<ITextRangeProvider>,
        other_endpoint: TextPatternRangeEndpoint,
    ) -> Result<i32> {
        let other = required_param(other)?.as_impl();
        self.read(|range| {
            other.read(|other_range| {
                if range.node().id() != other_range.node().id() {
                    return Err(invalid_arg());
                }
                let pos = position_from_endpoint(range, endpoint)?;
                let other_pos = position_from_endpoint(other_range, other_endpoint)?;
                let result = pos.partial_cmp(&other_pos).unwrap();
                Ok(result as i32)
            })
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
        todo!()
    }

    fn MoveEndpointByRange(
        &self,
        endpoint: TextPatternRangeEndpoint,
        other: &Option<ITextRangeProvider>,
        other_endpoint: TextPatternRangeEndpoint,
    ) -> Result<()> {
        todo!()
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
