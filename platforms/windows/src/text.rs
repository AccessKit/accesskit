// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit_consumer::{TextRange as Range, Tree, WeakTextRange as WeakRange};
use parking_lot::RwLock;
use std::sync::Weak;
use windows::{
    core::*,
    Win32::{Foundation::*, System::Com::*, UI::Accessibility::*},
};

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
}

impl Clone for PlatformRange {
    fn clone(&self) -> Self {
        PlatformRange {
            tree: self.tree.clone(),
            state: RwLock::new(*self.state.read()),
        }
    }
}

impl ITextRangeProvider_Impl for PlatformRange {
    fn Clone(&self) -> Result<ITextRangeProvider> {
        Ok(self.clone().into())
    }

    fn Compare(&self, other: &Option<ITextRangeProvider>) -> Result<BOOL> {
        todo!()
    }

    fn CompareEndpoints(
        &self,
        endpoint: TextPatternRangeEndpoint,
        other: &Option<ITextRangeProvider>,
        other_endpoint: TextPatternRangeEndpoint,
    ) -> Result<i32> {
        todo!()
    }

    fn ExpandToEnclosingUnit(&self, unit: TextUnit) -> Result<()> {
        todo!()
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
