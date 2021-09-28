// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit_consumer::{Node, WeakNode};
use accesskit_windows_bindings::Windows::Win32::{
    Foundation::*, System::OleAutomation::*, UI::Accessibility::*,
};
use accesskit_windows_bindings::*;
use windows::*;

use crate::util::*;

#[implement(Windows::Win32::UI::Accessibility::IRawElementProviderSimple)]
pub(crate) struct PlatformNode {
    node: WeakNode,
    hwnd: HWND,
}

#[allow(non_snake_case, non_upper_case_globals)]
impl PlatformNode {
    pub(crate) fn new(node: &Node, hwnd: HWND) -> PlatformNode {
        Self {
            node: node.downgrade(),
            hwnd: hwnd,
        }
    }

    fn resolve<F, T>(&self, f: F) -> Result<T>
    where
        for<'a> F: FnOnce(&Node<'a>) -> Result<T>,
    {
        self.node
            .map(f)
            .unwrap_or_else(|| Err(Error::new(HRESULT(UIA_E_ELEMENTNOTAVAILABLE), "")))
    }

    fn ProviderOptions(&self) -> Result<ProviderOptions> {
        // We don't currently have to resolve the node to implement this.
        // But we might have to in the future. So to avoid leaking
        // implementation details that might change, we'll resolve
        // the node and just ignore it. There's precedent for this;
        // Chromium's implementation of this method validates the node
        // even though the return value is hard-coded.
        self.resolve(|_node| Ok(ProviderOptions_ServerSideProvider))
    }

    fn GetPatternProvider(&self, _pattern_id: i32) -> Result<IUnknown> {
        self.resolve(|_node| {
            // TODO: add patterns
            Err(Error::OK)
        })
    }

    fn GetPropertyValue(&self, property_id: i32) -> Result<VARIANT> {
        self.resolve(|node| {
            // TODO: add properties
            match property_id {
                UIA_NamePropertyId => {
                    if let Some(name) = node.name() {
                        return Ok(variant_from_bstr(name.into()));
                    }
                }
                _ => (),
            }
            Ok(empty_variant())
        })
    }

    fn HostRawElementProvider(&self) -> Result<IRawElementProviderSimple> {
        self.resolve(|node| {
            if node.is_root() {
                unsafe { UiaHostProviderFromHwnd(self.hwnd) }
            } else {
                Err(Error::OK)
            }
        })
    }
}
