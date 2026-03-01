// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

//! Localized accessibility strings for AccessKit.
//!
//! Provides localized role descriptions and state descriptions for accessibility
//! that lack native localized equivalents on some platforms. Platform adapters
//! call into this crate where the OS accessibility API doesn't provide adequate
//! localized names.

#![no_std]

extern crate alloc;

use accesskit::Role;
use alloc::string::String;

/// Keys for localized state description strings.
///
/// These correspond to accessibility state information that must be conveyed
/// as free-text strings on platforms like Android, where typed state properties
/// are not available for all state types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum StateKey {
    ToggleButtonOn,
    ToggleButtonOff,
    AriaCurrentPage,
    AriaCurrentStep,
    AriaCurrentLocation,
    AriaCurrentDate,
    AriaCurrentTime,
    AriaCurrentTrue,
    LinkVisited,
    InvalidTrue,
    InvalidSpelling,
    InvalidGrammar,
    MultiselectableNone,
}

include!("generated.rs");

/// A resolved locale, ready for localized string lookups.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocaleId(usize);

impl LocaleId {
    /// Resolve a locale tag to the best available translations.
    ///
    /// Accepts a BCP 47 tag such as `"en-US"`, `"pt-BR"`, or `"zh-Hant-TW"`;
    /// POSIX-style locales like `"en_US.UTF-8"` are also accepted. Regional and
    /// script variants are matched where translations differentiate them (for
    /// example, `"en-AU"` uses British English and `"zh-TW"` uses Traditional
    /// Chinese). An unrecognized locale resolves to English.
    pub fn new(locale: &str) -> Self {
        let normalized = normalize(locale);
        let tag = canonical_tag(&normalized);

        let mut candidate = tag;
        loop {
            if let Some(index) = locale_index(candidate) {
                return Self(index);
            }
            match candidate.rfind('-') {
                Some(pos) => candidate = &candidate[..pos],
                None => break,
            }
        }

        Self(locale_index("en").expect("English translations must be present"))
    }
}

fn canonical_tag(tag: &str) -> &str {
    match tag {
        "zh-tw" => "zh-hant-tw",
        "zh-hk" => "zh-hant-hk",
        "zh-mo" => "zh-hant-mo",
        _ => match tag.split_once('-') {
            Some(("en", region)) if region != "us" => "en-gb",
            Some(("es", region)) if !matches!(region, "es" | "gq" | "ea" | "ic" | "ph") => "es-419",
            _ => tag,
        },
    }
}

/// Look up a localized role description (no interpolation).
///
/// Returns `None` if this role doesn't have a localized description.
pub fn role_description(locale: LocaleId, role: Role) -> Option<&'static str> {
    lookup_impl(locale.0, role)
}

/// Look up a localized state description (no interpolation).
///
/// Returns `None` if this state key doesn't have a localized description.
pub fn state_description(locale: LocaleId, key: StateKey) -> Option<&'static str> {
    state_lookup_impl(locale.0, key)
}

/// Render a localized [`InterpolatedString`] into `out`.
pub fn write_interpolated_string(
    locale: LocaleId,
    string: InterpolatedString<'_>,
    out: &mut dyn core::fmt::Write,
) -> core::fmt::Result {
    write_interpolated_string_impl(locale.0, string, out)
}

fn normalize(locale: &str) -> String {
    let without_encoding = locale.split('.').next().unwrap_or("");
    without_encoding.replace('_', "-").to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Locale normalization tests ---

    #[test]
    fn normalize_bcp47() {
        assert_eq!(normalize("en-US"), "en-us");
    }

    #[test]
    fn normalize_posix_underscores() {
        assert_eq!(normalize("en_US"), "en-us");
    }

    #[test]
    fn normalize_posix_encoding_suffix() {
        assert_eq!(normalize("en_US.UTF-8"), "en-us");
    }

    #[test]
    fn normalize_case() {
        assert_eq!(normalize("EN-US"), "en-us");
    }

    #[test]
    fn normalize_empty() {
        assert_eq!(normalize(""), "");
    }

    #[test]
    fn normalize_script_tag() {
        assert_eq!(normalize("zh-Hans-CN"), "zh-hans-cn");
    }

    // --- Locale resolution tests ---

    #[test]
    fn resolve_region_falls_back_to_language() {
        // A region with no specific variant resolves to the bare language.
        assert_eq!(LocaleId::new("fr-BE"), LocaleId::new("fr"));
        assert_eq!(LocaleId::new("pt-AO"), LocaleId::new("pt"));
        assert_eq!(LocaleId::new("de-AT"), LocaleId::new("de"));
    }

    #[test]
    fn resolve_english_region_variants() {
        // The US (and bare `en`) use American English; every other region uses
        // British.
        let en = LocaleId::new("en");
        let gb = LocaleId::new("en-GB");
        assert_ne!(en, gb);
        assert_eq!(LocaleId::new("en-US"), en);
        assert_eq!(LocaleId::new("en-AU"), gb);
        assert_eq!(LocaleId::new("en-NZ"), gb);
        assert_eq!(LocaleId::new("en-IE"), gb);
        assert_eq!(LocaleId::new("en-IN"), gb);
    }

    #[test]
    fn resolve_spanish_region_variants() {
        // Spain uses European Spanish; the Americas use Latin American Spanish.
        let es = LocaleId::new("es");
        let la = LocaleId::new("es-419");
        assert_ne!(es, la);
        assert_eq!(LocaleId::new("es-ES"), es);
        assert_eq!(LocaleId::new("es-MX"), la);
        assert_eq!(LocaleId::new("es-AR"), la);
        assert_eq!(LocaleId::new("es-US"), la);
    }

    #[test]
    fn resolve_chinese_simplified() {
        // Bare `zh`, region tags, and explicit `Hans` all reach Simplified.
        let zh = LocaleId::new("zh");
        assert_eq!(LocaleId::new("zh-CN"), zh);
        assert_eq!(LocaleId::new("zh-SG"), zh);
        assert_eq!(LocaleId::new("zh-Hans"), zh);
        assert_eq!(LocaleId::new("zh-Hans-CN"), zh);
    }

    #[test]
    fn resolve_chinese_traditional() {
        // Desktop region tags (`zh-TW`) and mobile script tags (`zh-Hant-TW`)
        // both reach Traditional, distinct from Simplified.
        let hant = LocaleId::new("zh-Hant");
        assert_eq!(LocaleId::new("zh-TW"), hant);
        assert_eq!(LocaleId::new("zh-Hant-TW"), hant);
        assert_ne!(hant, LocaleId::new("zh"));

        // Hong Kong Traditional is its own row.
        let hk = LocaleId::new("zh-HK");
        assert_eq!(LocaleId::new("zh-Hant-HK"), hk);
        assert_ne!(hk, hant);
    }

    #[test]
    fn resolve_serbian_script() {
        // Latin is explicit; the default and region-only tags are Cyrillic.
        let latin = LocaleId::new("sr-Latn");
        assert_eq!(LocaleId::new("sr-Latn-RS"), latin);
        let cyrillic = LocaleId::new("sr");
        assert_eq!(LocaleId::new("sr-RS"), cyrillic);
        assert_ne!(latin, cyrillic);
    }

    // --- Interpolated string tests ---

    fn interpolated(locale: &str, string: InterpolatedString) -> String {
        let mut buf = String::new();
        write_interpolated_string(LocaleId::new(locale), string, &mut buf).unwrap();
        buf
    }

    #[test]
    fn interpolated_heading_level() {
        assert_eq!(
            interpolated("en", InterpolatedString::HeadingWithLevel("2")),
            "heading level 2"
        );
    }

    #[test]
    fn interpolated_unknown_locale_uses_english() {
        assert_eq!(
            interpolated("xx", InterpolatedString::HeadingWithLevel("3")),
            "heading level 3"
        );
    }

    #[test]
    fn interpolated_multiple_args() {
        assert_eq!(
            interpolated("en", InterpolatedString::Multiselectable("3", "10")),
            "multiselectable, 3 of 10 selected."
        );
    }

    #[test]
    fn interpolated_radio() {
        assert_eq!(
            interpolated("en", InterpolatedString::RadioChecked("2", "5")),
            "Checked. In group, option 2 of 5"
        );
        assert_eq!(
            interpolated("en", InterpolatedString::RadioUnchecked("1", "3")),
            "Not checked. In group, option 1 of 3"
        );
    }

    #[test]
    fn interpolated_non_english_locale() {
        // A non-English locale still interpolates the argument.
        let de = interpolated("de", InterpolatedString::HeadingWithLevel("2"));
        assert!(de.contains('2'), "argument should be interpolated: {de:?}");
    }

    // --- Integration tests using generated code ---

    #[test]
    fn role_description_english() {
        let en = LocaleId::new("en");
        assert_eq!(role_description(en, Role::Banner), Some("banner"));
        assert_eq!(
            role_description(en, Role::ContentInfo),
            Some("content information")
        );
        assert_eq!(role_description(en, Role::DocAbstract), Some("abstract"));
    }

    #[test]
    fn role_description_not_localized() {
        // Button is handled by the platform, not by l10n
        let en = LocaleId::new("en");
        assert_eq!(role_description(en, Role::Button), None);
        assert_eq!(role_description(en, Role::CheckBox), None);
    }

    #[test]
    fn role_description_unknown_locale_falls_back_to_english() {
        assert_eq!(
            role_description(LocaleId::new("xx-YY"), Role::Banner),
            Some("banner")
        );
    }

    #[test]
    fn role_description_empty_locale_falls_back_to_english() {
        assert_eq!(
            role_description(LocaleId::new(""), Role::Banner),
            Some("banner")
        );
    }

    #[test]
    fn all_landmark_roles_have_descriptions() {
        let en = LocaleId::new("en");
        let landmarks = [
            Role::Banner,
            Role::Complementary,
            Role::ContentInfo,
            Role::Main,
            Role::Navigation,
            Role::Region,
            Role::Search,
        ];
        for role in landmarks {
            assert!(
                role_description(en, role).is_some(),
                "landmark role {role:?} should have a description"
            );
        }
    }

    #[test]
    fn all_dpub_roles_have_descriptions() {
        let en = LocaleId::new("en");
        let dpub_roles = [
            Role::DocAbstract,
            Role::DocAcknowledgements,
            Role::DocAfterword,
            Role::DocAppendix,
            Role::DocBackLink,
            Role::DocBiblioEntry,
            Role::DocBibliography,
            Role::DocBiblioRef,
            Role::DocChapter,
            Role::DocColophon,
            Role::DocConclusion,
            Role::DocCover,
            Role::DocCredit,
            Role::DocCredits,
            Role::DocDedication,
            Role::DocEndnote,
            Role::DocEndnotes,
            Role::DocEpigraph,
            Role::DocEpilogue,
            Role::DocErrata,
            Role::DocExample,
            Role::DocFootnote,
            Role::DocForeword,
            Role::DocGlossary,
            Role::DocGlossRef,
            Role::DocIndex,
            Role::DocIntroduction,
            Role::DocNoteRef,
            Role::DocNotice,
            Role::DocPageBreak,
            Role::DocPageFooter,
            Role::DocPageHeader,
            Role::DocPageList,
            Role::DocPart,
            Role::DocPreface,
            Role::DocPrologue,
            Role::DocPullquote,
            Role::DocQna,
            Role::DocSubtitle,
            Role::DocTip,
            Role::DocToc,
        ];
        for role in dpub_roles {
            assert!(
                role_description(en, role).is_some(),
                "dpub role {role:?} should have a description"
            );
        }
    }

    // --- State description tests ---

    #[test]
    fn state_description_static_english() {
        let en = LocaleId::new("en");
        assert_eq!(state_description(en, StateKey::ToggleButtonOn), Some("On"));
        assert_eq!(
            state_description(en, StateKey::ToggleButtonOff),
            Some("Off")
        );
        assert_eq!(
            state_description(en, StateKey::AriaCurrentPage),
            Some("current page")
        );
        assert_eq!(
            state_description(en, StateKey::LinkVisited),
            Some("visited")
        );
        assert_eq!(
            state_description(en, StateKey::InvalidTrue),
            Some("Invalid entry")
        );
        assert_eq!(
            state_description(en, StateKey::InvalidSpelling),
            Some("Invalid spelling")
        );
        assert_eq!(
            state_description(en, StateKey::InvalidGrammar),
            Some("Invalid grammar")
        );
        assert_eq!(
            state_description(en, StateKey::MultiselectableNone),
            Some("multiselectable, none selected.")
        );
    }

    #[test]
    fn state_description_fallback_to_english() {
        assert_eq!(
            state_description(LocaleId::new("xx-YY"), StateKey::ToggleButtonOn),
            Some("On")
        );
    }

    #[test]
    fn all_static_state_keys_have_descriptions() {
        let en = LocaleId::new("en");
        let static_keys = [
            StateKey::ToggleButtonOn,
            StateKey::ToggleButtonOff,
            StateKey::AriaCurrentPage,
            StateKey::AriaCurrentStep,
            StateKey::AriaCurrentLocation,
            StateKey::AriaCurrentDate,
            StateKey::AriaCurrentTime,
            StateKey::AriaCurrentTrue,
            StateKey::LinkVisited,
            StateKey::InvalidTrue,
            StateKey::InvalidSpelling,
            StateKey::InvalidGrammar,
            StateKey::MultiselectableNone,
        ];
        for key in static_keys {
            assert!(
                state_description(en, key).is_some(),
                "state key {key:?} should have a description"
            );
        }
    }
}
