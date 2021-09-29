// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

#![allow(non_upper_case_globals)]

use accesskit_consumer::{Node, WeakNode};
use accesskit_windows_bindings::Windows::Win32::{
    Foundation::*, System::OleAutomation::*, UI::Accessibility::*,
};
use accesskit_windows_bindings::*;
use windows::*;

use crate::util::*;

struct ResolvedPlatformNode<'a> {
    node: &'a Node<'a>,
    hwnd: HWND,
}

impl ResolvedPlatformNode<'_> {
    fn provider_options(&self) -> ProviderOptions {
        ProviderOptions_ServerSideProvider
    }

    fn is_pattern_supported(&self, _pattern_id: i32) -> bool {
        // TODO: add patterns
        false
    }

    fn get_property_value(&self, property_id: i32) -> VARIANT {
        // TODO: add properties
        match property_id {
            UIA_NamePropertyId => {
                if let Some(name) = self.node.name() {
                    return variant_from_bstr(name.into());
                }
            }
            _ => (),
        }
        empty_variant()
    }

    fn host_provider(&self) -> Result<IRawElementProviderSimple> {
        if self.node.is_root() {
            unsafe { UiaHostProviderFromHwnd(self.hwnd) }
        } else {
            Err(Error::OK)
        }
    }
}

#[implement(
    Windows::Win32::UI::Accessibility::IRawElementProviderSimple,
    Windows::Win32::UI::Accessibility::IRawElementProviderFragment
)]
pub(crate) struct PlatformNode {
    node: WeakNode,
    hwnd: HWND,
}

#[allow(non_snake_case)]
impl PlatformNode {
    pub(crate) fn new(node: &Node, hwnd: HWND) -> PlatformNode {
        Self {
            node: node.downgrade(),
            hwnd,
        }
    }

    fn resolve<F, T>(&self, f: F) -> Result<T>
    where
        for<'a> F: FnOnce(ResolvedPlatformNode<'a>) -> Result<T>,
    {
        self.node
            .map(|node| {
                f(ResolvedPlatformNode {
                    node,
                    hwnd: self.hwnd,
                })
            })
            .unwrap_or_else(|| Err(Error::new(HRESULT(UIA_E_ELEMENTNOTAVAILABLE), "")))
    }

    fn ProviderOptions(&self) -> Result<ProviderOptions> {
        // We don't currently have to resolve the node to implement this.
        // But we might have to in the future. So to avoid leaking
        // implementation details that might change, we'll resolve
        // the node and just ignore it. There's precedent for this;
        // Chromium's implementation of this method validates the node
        // even though the return value is hard-coded.
        self.resolve(|resolved| Ok(resolved.provider_options()))
    }

    fn GetPatternProvider(&mut self, pattern_id: i32) -> Result<IUnknown> {
        let supported = self.resolve(|resolved| Ok(resolved.is_pattern_supported(pattern_id)))?;
        if supported {
            let intermediate: IRawElementProviderSimple = self.into();
            Ok(intermediate.into())
        } else {
            Err(Error::OK)
        }
    }

    fn GetPropertyValue(&self, property_id: i32) -> Result<VARIANT> {
        self.resolve(|resolved| Ok(resolved.get_property_value(property_id)))
    }

    fn HostRawElementProvider(&self) -> Result<IRawElementProviderSimple> {
        self.resolve(|resolved| resolved.host_provider())
    }

    fn Navigate(&self, _direction: NavigateDirection) -> Result<IRawElementProviderFragment> {
        unimplemented!()
    }

    fn GetRuntimeId(&self) -> Result<*mut SAFEARRAY> {
        unimplemented!()
    }

    fn BoundingRectangle(&self) -> Result<UiaRect> {
        unimplemented!()
    }

    fn GetEmbeddedFragmentRoots(&self) -> Result<*mut SAFEARRAY> {
        unimplemented!()
    }

    fn SetFocus(&self) -> Result<()> {
        unimplemented!()
    }

    fn FragmentRoot(&self) -> Result<IRawElementProviderFragmentRoot> {
        unimplemented!()
    }
}
