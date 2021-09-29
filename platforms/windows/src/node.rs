// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

#![allow(non_upper_case_globals)]

use accesskit_consumer::{Node, WeakNode};
use accesskit_schema::NodeIdContent;
use accesskit_windows_bindings::Windows::Win32::{
    Foundation::*, Graphics::Gdi::*, System::OleAutomation::*, UI::Accessibility::*,
};
use accesskit_windows_bindings::*;
use arrayvec::ArrayVec;
use windows::*;

use crate::util::*;

struct ResolvedPlatformNode<'a> {
    node: Node<'a>,
    hwnd: HWND,
}

impl ResolvedPlatformNode<'_> {
    fn new(node: Node, hwnd: HWND) -> ResolvedPlatformNode {
        ResolvedPlatformNode { node, hwnd }
    }

    fn relative<'a>(&self, node: Node<'a>) -> ResolvedPlatformNode<'a> {
        ResolvedPlatformNode::new(node, self.hwnd)
    }

    fn downgrade(&self) -> PlatformNode {
        PlatformNode::new(&self.node, self.hwnd)
    }

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

    fn navigate(&self, direction: NavigateDirection) -> Option<ResolvedPlatformNode> {
        let result = match direction {
            NavigateDirection_Parent => self.node.unignored_parent(),
            NavigateDirection_NextSibling => self.node.following_unignored_siblings().next(),
            NavigateDirection_PreviousSibling => self.node.preceding_unignored_siblings().next(),
            NavigateDirection_FirstChild => self.node.unignored_children().next(),
            NavigateDirection_LastChild => self.node.unignored_children().next_back(),
            _ => None,
        };
        result.map(|node| self.relative(node))
    }

    fn runtime_id(&self) -> impl std::ops::Deref<Target = [i32]> {
        let mut result = ArrayVec::<i32, { std::mem::size_of::<NodeIdContent>() + 1 }>::new();
        result.push(UiaAppendRuntimeId as i32);
        let id = self.node.id().0;
        let id_bytes = id.get().to_be_bytes();
        let start_index: usize = (id.leading_zeros() / 8) as usize;
        for byte in &id_bytes[start_index..] {
            result.push((*byte).into());
        }
        result
    }

    fn bounding_rectangle(&self) -> UiaRect {
        self.node.bounds().map_or(UiaRect::default(), |rect| {
            let mut result = UiaRect {
                left: rect.left.into(),
                top: rect.top.into(),
                width: rect.width.into(),
                height: rect.height.into(),
            };
            let mut client_top_left = POINT::default();
            unsafe { ClientToScreen(self.hwnd, &mut client_top_left) }.unwrap();
            result.left += f64::from(client_top_left.x);
            result.top += f64::from(client_top_left.y);
            result
        })
    }

    fn set_focus(&self) {
        // TODO: request action
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
    pub(crate) fn new(node: &Node, hwnd: HWND) -> Self {
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
            .map(|node| f(ResolvedPlatformNode::new(node, self.hwnd)))
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

    fn Navigate(&self, direction: NavigateDirection) -> Result<IRawElementProviderFragment> {
        self.resolve(|resolved| match resolved.navigate(direction) {
            Some(result) => Ok(result.downgrade().into()),
            None => Err(Error::OK),
        })
    }

    fn GetRuntimeId(&self) -> Result<*mut SAFEARRAY> {
        self.resolve(|resolved| {
            let runtime_id = resolved.runtime_id();
            Ok(safe_array_from_i32_slice(&runtime_id))
        })
    }

    fn BoundingRectangle(&self) -> Result<UiaRect> {
        self.resolve(|resolved| Ok(resolved.bounding_rectangle()))
    }

    fn GetEmbeddedFragmentRoots(&self) -> Result<*mut SAFEARRAY> {
        // As with ProviderOptions above, avoid leaking implementation details.
        self.resolve(|_resolved| Ok(std::ptr::null_mut()))
    }

    fn SetFocus(&self) -> Result<()> {
        self.resolve(|resolved| {
            resolved.set_focus();
            Ok(())
        })
    }

    fn FragmentRoot(&self) -> Result<IRawElementProviderFragmentRoot> {
        unimplemented!()
    }
}
