// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use objc2::foundation::{NSInteger, NSString};

#[link(name = "AppKit", kind = "framework")]
extern "C" {
    // Notifications
    pub(crate) static NSAccessibilityUIElementDestroyedNotification: &'static NSString;
    pub(crate) static NSAccessibilityFocusedUIElementChangedNotification: &'static NSString;
    pub(crate) static NSAccessibilityTitleChangedNotification: &'static NSString;
    pub(crate) static NSAccessibilityValueChangedNotification: &'static NSString;
    pub(crate) static NSAccessibilitySelectedTextChangedNotification: &'static NSString;
    pub(crate) static NSAccessibilityAnnouncementRequestedNotification: &'static NSString;

    // Roles
    pub(crate) static NSAccessibilityButtonRole: &'static NSString;
    pub(crate) static NSAccessibilityCheckBoxRole: &'static NSString;
    pub(crate) static NSAccessibilityCellRole: &'static NSString;
    pub(crate) static NSAccessibilityColorWellRole: &'static NSString;
    pub(crate) static NSAccessibilityColumnRole: &'static NSString;
    pub(crate) static NSAccessibilityComboBoxRole: &'static NSString;
    pub(crate) static NSAccessibilityGroupRole: &'static NSString;
    pub(crate) static NSAccessibilityImageRole: &'static NSString;
    pub(crate) static NSAccessibilityIncrementorRole: &'static NSString;
    pub(crate) static NSAccessibilityLevelIndicatorRole: &'static NSString;
    pub(crate) static NSAccessibilityLinkRole: &'static NSString;
    pub(crate) static NSAccessibilityListRole: &'static NSString;
    pub(crate) static NSAccessibilityMenuRole: &'static NSString;
    pub(crate) static NSAccessibilityMenuBarRole: &'static NSString;
    pub(crate) static NSAccessibilityMenuItemRole: &'static NSString;
    pub(crate) static NSAccessibilityOutlineRole: &'static NSString;
    pub(crate) static NSAccessibilityPopUpButtonRole: &'static NSString;
    pub(crate) static NSAccessibilityProgressIndicatorRole: &'static NSString;
    pub(crate) static NSAccessibilityRadioButtonRole: &'static NSString;
    pub(crate) static NSAccessibilityRadioGroupRole: &'static NSString;
    pub(crate) static NSAccessibilityRowRole: &'static NSString;
    pub(crate) static NSAccessibilityScrollBarRole: &'static NSString;
    pub(crate) static NSAccessibilitySliderRole: &'static NSString;
    pub(crate) static NSAccessibilitySplitterRole: &'static NSString;
    pub(crate) static NSAccessibilityStaticTextRole: &'static NSString;
    pub(crate) static NSAccessibilityTabGroupRole: &'static NSString;
    pub(crate) static NSAccessibilityTableRole: &'static NSString;
    pub(crate) static NSAccessibilityTextAreaRole: &'static NSString;
    pub(crate) static NSAccessibilityTextFieldRole: &'static NSString;
    pub(crate) static NSAccessibilityToolbarRole: &'static NSString;
    pub(crate) static NSAccessibilityUnknownRole: &'static NSString;

    // Notification user info keys
    pub(crate) static NSAccessibilityAnnouncementKey: &'static NSString;
    pub(crate) static NSAccessibilityPriorityKey: &'static NSString;
}

// Announcement priorities
pub(crate) const NSAccessibilityPriorityMedium: NSInteger = 50;
pub(crate) const NSAccessibilityPriorityHigh: NSInteger = 90;
