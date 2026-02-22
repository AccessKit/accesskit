// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

//! Localization code generator for AccessKit.
//!
//! This crate parses simple key=value translation files and generates the
//! Rust lookup code for the `accesskit_l10n` runtime crate. The generated code
//! is committed to the repository (`l10n/src/generated.rs`) rather than built
//! at compile time; run the crate's binary (`cargo run -p accesskit_l10n_build`)
//! to regenerate it after editing the translation files.

use heck::{ToShoutySnakeCase, ToUpperCamelCase};
use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;

/// A parsed translation file for a single locale.
#[derive(Debug, Clone)]
pub struct TranslationFile {
    pub locale: String,
    pub entries: BTreeMap<String, String>,
}

/// Errors that can occur during parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// A line has no `=` delimiter.
    MissingDelimiter { line_number: usize, line: String },
    /// A key is empty (after trimming).
    EmptyKey { line_number: usize },
    /// A key appears more than once.
    DuplicateKey { line_number: usize, key: String },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::MissingDelimiter { line_number, line } => {
                write!(f, "line {line_number}: missing '=' delimiter: {line}")
            }
            ParseError::EmptyKey { line_number } => {
                write!(f, "line {line_number}: empty key")
            }
            ParseError::DuplicateKey { line_number, key } => {
                write!(f, "line {line_number}: duplicate key '{key}'")
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// Result of validating a set of translation files.
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl ValidationResult {
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

/// Parse a single translation file from its content string.
///
/// Format:
/// - Lines starting with `#` are comments
/// - Empty lines are ignored
/// - Format: `key = value` (first `=` is delimiter)
/// - `\r\n` and `\n` both accepted
pub fn parse_translation_file(locale: &str, content: &str) -> Result<TranslationFile, ParseError> {
    let mut entries = BTreeMap::new();

    for (line_idx, line) in content.lines().enumerate() {
        let line_number = line_idx + 1;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let Some(eq_pos) = trimmed.find('=') else {
            return Err(ParseError::MissingDelimiter {
                line_number,
                line: trimmed.to_string(),
            });
        };

        let key = trimmed[..eq_pos].trim();
        let value = trimmed[eq_pos + 1..].trim();

        if key.is_empty() {
            return Err(ParseError::EmptyKey { line_number });
        }

        if entries.contains_key(key) {
            return Err(ParseError::DuplicateKey {
                line_number,
                key: key.to_string(),
            });
        }

        entries.insert(key.to_string(), value.to_string());
    }

    Ok(TranslationFile {
        locale: locale.to_string(),
        entries,
    })
}

fn extract_placeholders(value: &str) -> BTreeSet<usize> {
    let mut placeholders = BTreeSet::new();
    let bytes = value.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                i += 2;
                continue;
            }
            if let Some(close) = value[i..].find('}') {
                let inner = &value[i + 1..i + close];
                if let Ok(idx) = inner.parse::<usize>() {
                    placeholders.insert(idx);
                }
                i += close + 1;
                continue;
            }
        }
        i += 1;
    }
    placeholders
}

/// Validate a set of translation files.
///
/// Rules:
/// - There must be an "en" (English) file (error if missing)
/// - Every key in any non-English file must exist in English (error if not)
/// - Non-English files missing keys from English produce warnings
/// - Placeholder indices in non-English values must match English (error if not)
pub fn validate(files: &[TranslationFile]) -> ValidationResult {
    let mut result = ValidationResult::default();

    let english = files.iter().find(|f| f.locale == "en");
    let Some(english) = english else {
        result
            .errors
            .push("missing required English translation file (en.txt)".to_string());
        return result;
    };

    if english.entries.is_empty() {
        result
            .errors
            .push("English translation file is empty".to_string());
        return result;
    }

    let english_keys: BTreeSet<&str> = english.entries.keys().map(|s| s.as_str()).collect();

    for file in files {
        if file.locale == "en" {
            continue;
        }

        let file_keys: BTreeSet<&str> = file.entries.keys().map(|s| s.as_str()).collect();

        for key in file_keys.difference(&english_keys) {
            result.errors.push(format!(
                "{}: key '{}' not found in English file",
                file.locale, key
            ));
        }

        for key in english_keys.difference(&file_keys) {
            result.warnings.push(format!(
                "{}: missing key '{}' (will fall back to English)",
                file.locale, key
            ));
        }

        for key in english_keys.intersection(&file_keys) {
            let en_value = &english.entries[*key];
            let other_value = &file.entries[*key];
            let en_placeholders = extract_placeholders(en_value);
            let other_placeholders = extract_placeholders(other_value);

            if en_placeholders != other_placeholders {
                result.errors.push(format!(
                    "{}: key '{}' has placeholders {:?} but English has {:?}",
                    file.locale, key, other_placeholders, en_placeholders
                ));
            }
        }
    }

    result
}

fn key_to_role_variant(key: &str) -> Option<String> {
    if key == "heading_with_level" {
        return Some("Heading".to_string());
    }
    Some(key.strip_prefix("role.")?.to_upper_camel_case())
}

fn key_to_state_variant(key: &str) -> Option<String> {
    Some(key.strip_prefix("state.")?.to_upper_camel_case())
}

fn string_literal(value: &str) -> Literal {
    let mut repr = String::with_capacity(value.len() + 2);
    repr.push('"');
    for ch in value.chars() {
        match ch {
            '"' | '\\' => repr.extend(ch.escape_default()),
            ch if ch.is_control() => repr.extend(ch.escape_default()),
            ch => repr.push(ch),
        }
    }
    repr.push('"');
    repr.parse().expect("string literal should parse")
}

fn strip_region(locale: &str) -> Option<&str> {
    let first_dash = locale.find('-')?;
    let after_first = &locale[first_dash + 1..];
    match after_first.find('-') {
        None => {
            // Two subtags: a 4-letter second subtag is a script (keep it),
            // anything else is a region (strip it).
            if after_first.len() == 4 {
                None
            } else {
                Some(&locale[..first_dash])
            }
        }
        Some(second_dash_offset) => {
            let second_dash = first_dash + 1 + second_dash_offset;
            Some(&locale[..second_dash])
        }
    }
}

fn base_language<'a>(locale: &str, locales: &BTreeSet<&'a str>) -> Option<&'a str> {
    locales.get(strip_region(locale)?).copied()
}

fn generate_lookup_fn(
    fn_name: &str,
    binding: &str,
    variant_type: &str,
    entries: &[(String, String, String)],
) -> TokenStream {
    let fn_name = format_ident!("{fn_name}");
    let binding = format_ident!("{binding}");
    let variant_type = format_ident!("{variant_type}");
    let arms = entries.iter().map(|(locale, variant, value)| {
        let locale = format_ident!("{}", locale.to_shouty_snake_case());
        let variant = format_ident!("{variant}");
        let value = string_literal(value);
        quote! { (#locale, #variant_type::#variant) => Some(#value), }
    });

    // Some translations legitimately contain zero-width word-break characters
    // (e.g. U+200B in Lao and Burmese), which clippy's `invisible_characters`
    // lint would reject, so the generated functions allow it.
    quote! {
        #[allow(clippy::invisible_characters)]
        pub(crate) fn #fn_name(locale: usize, #binding: #variant_type) -> Option<&'static str> {
            match (locale, #binding) {
                #(#arms)*
                _ => None,
            }
        }
    }
}

fn generate_locale_support(locales: &[String]) -> TokenStream {
    let tags = locales.iter().map(|locale| string_literal(locale));
    let consts = locales.iter().enumerate().map(|(index, locale)| {
        let name = format_ident!("{}", locale.to_shouty_snake_case());
        let index = Literal::usize_unsuffixed(index);
        quote! { const #name: usize = #index; }
    });

    quote! {
        /// The locale tags that have translations, sorted for binary search. Each tag's
        /// index is the key used by the lookup functions below.
        static LOCALE_TAGS: &[&str] = &[#(#tags),*];

        /// Resolve a locale tag to its index in `LOCALE_TAGS`, or `None` if the tag has
        /// no translations.
        pub(crate) fn locale_index(locale: &str) -> Option<usize> {
            LOCALE_TAGS.binary_search(&locale).ok()
        }

        #(#consts)*
    }
}

fn key_to_message_variant(key: &str) -> String {
    let base = key
        .strip_prefix("state.")
        .or_else(|| key.strip_prefix("role."))
        .unwrap_or(key);
    base.to_upper_camel_case()
}

fn template_arity(placeholders: &BTreeSet<usize>, key: &str) -> usize {
    let arity = placeholders.len();
    for i in 0..arity {
        assert!(
            placeholders.contains(&i),
            "key '{key}': interpolation placeholders must be contiguous 0..{arity}, got {placeholders:?}"
        );
    }
    arity
}

fn rewrite_placeholders(template: &str) -> String {
    let mut result = String::with_capacity(template.len());
    let bytes = template.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            if bytes.get(i + 1) == Some(&b'{') {
                result.push_str("{{");
                i += 2;
                continue;
            }
            if let Some(close) = template[i..].find('}') {
                if let Ok(idx) = template[i + 1..i + close].parse::<usize>() {
                    result.push_str(&format!("{{a{idx}}}"));
                    i += close + 1;
                    continue;
                }
            }
            result.push('{');
            i += 1;
        } else if bytes[i] == b'}' && bytes.get(i + 1) == Some(&b'}') {
            result.push_str("}}");
            i += 2;
        } else {
            let ch = template[i..].chars().next().unwrap();
            result.push(ch);
            i += ch.len_utf8();
        }
    }
    result
}

fn effective_value<'a>(
    entries_by_locale: &BTreeMap<&str, &'a BTreeMap<String, String>>,
    english: &'a BTreeMap<String, String>,
    locale_set: &BTreeSet<&str>,
    locale: &str,
    key: &str,
) -> &'a str {
    let own = entries_by_locale[locale].get(key);
    let base = base_language(locale, locale_set).and_then(|base| entries_by_locale[base].get(key));
    own.or(base).unwrap_or_else(|| &english[key]).as_str()
}

fn generate_interpolated(
    messages: &[(&str, String, usize)],
    locales: &[String],
    entries_by_locale: &BTreeMap<&str, &BTreeMap<String, String>>,
    english: &BTreeMap<String, String>,
    locale_set: &BTreeSet<&str>,
) -> TokenStream {
    let variants = messages.iter().map(|(_, variant, arity)| {
        let variant = format_ident!("{variant}");
        let fields = (0..*arity).map(|_| quote! { &'a str });
        quote! { #variant(#(#fields),*), }
    });

    let arms = messages.iter().map(|(key, variant, arity)| {
        let variant = format_ident!("{variant}");
        let bindings = (0..*arity).map(|i| format_ident!("a{i}"));
        let locale_arms = locales
            .iter()
            // English is emitted once, as the `_` default arm below.
            .filter(|locale| *locale != "en")
            .map(|locale| {
                let template = string_literal(&rewrite_placeholders(effective_value(
                    entries_by_locale,
                    english,
                    locale_set,
                    locale,
                    key,
                )));
                let locale = format_ident!("{}", locale.to_shouty_snake_case());
                quote! { #locale => write!(out, #template), }
            });
        let default = string_literal(&rewrite_placeholders(&english[*key]));
        quote! {
            InterpolatedString::#variant(#(#bindings),*) => match locale {
                #(#locale_arms)*
                _ => write!(out, #default),
            }
        }
    });

    quote! {
        /// An accessibility string with interpolated arguments.
        pub enum InterpolatedString<'a> {
            #(#variants)*
        }

        /// Render `string` for the locale at `locale` (a `LOCALE_TAGS` index) into `out`.
        #[allow(clippy::invisible_characters)]
        pub(crate) fn write_interpolated_string_impl(
            locale: usize,
            string: InterpolatedString<'_>,
            out: &mut dyn core::fmt::Write,
        ) -> core::fmt::Result {
            match string {
                #(#arms)*
            }
        }
    }
}

/// Generate Rust source code for the runtime lookups.
///
/// Produces the locale support code (see `generate_locale_support`), the two
/// static lookup functions (`lookup_impl` for roles, `state_lookup_impl` for
/// states), and the `InterpolatedString` enum plus its writer (see
/// `generate_interpolated`) for the templated strings.
///
/// English's keys define the universe of strings. Each locale's row is baked
/// with its own value where present, else its base language's, else English's,
/// so a lookup is a single index probe with no per-key fallback at runtime.
/// English is assumed present and non-empty (guaranteed by `validate`).
pub fn generate_code(files: &[TranslationFile]) -> String {
    let entries_by_locale: BTreeMap<&str, &BTreeMap<String, String>> = files
        .iter()
        .map(|f| (f.locale.as_str(), &f.entries))
        .collect();

    let mut locales: Vec<String> = files.iter().map(|f| f.locale.clone()).collect();
    locales.sort();
    locales.dedup();
    let locale_set: BTreeSet<&str> = locales.iter().map(String::as_str).collect();

    let english = entries_by_locale["en"];
    let mut role_static_keys: Vec<(&str, String)> = Vec::new();
    let mut state_static_keys: Vec<(&str, String)> = Vec::new();
    let mut messages: Vec<(&str, String, usize)> = Vec::new();
    for (key, en_value) in english {
        let placeholders = extract_placeholders(en_value);
        if placeholders.is_empty() {
            if let Some(variant) = key_to_role_variant(key) {
                role_static_keys.push((key, variant));
            } else if let Some(variant) = key_to_state_variant(key) {
                state_static_keys.push((key, variant));
            }
        } else if key_to_role_variant(key).is_some() || key_to_state_variant(key).is_some() {
            messages.push((
                key,
                key_to_message_variant(key),
                template_arity(&placeholders, key),
            ));
        }
    }
    role_static_keys.sort_by(|(_, a), (_, b)| a.cmp(b));
    state_static_keys.sort_by(|(_, a), (_, b)| a.cmp(b));
    messages.sort_by(|(_, a, _), (_, b, _)| a.cmp(b));

    let bake = |keys: &[(&str, String)]| -> Vec<(String, String, String)> {
        let mut entries = Vec::new();
        for locale in &locales {
            for (key, variant) in keys {
                let value = effective_value(&entries_by_locale, english, &locale_set, locale, key);
                entries.push((locale.clone(), variant.clone(), value.to_string()));
            }
        }
        entries
    };
    let static_entries = bake(&role_static_keys);
    let state_static_entries = bake(&state_static_keys);

    let locale_support = generate_locale_support(&locales);
    let lookup = generate_lookup_fn("lookup_impl", "role", "Role", &static_entries);
    let state_lookup = generate_lookup_fn(
        "state_lookup_impl",
        "key",
        "StateKey",
        &state_static_entries,
    );
    let interpolated = generate_interpolated(
        &messages,
        &locales,
        &entries_by_locale,
        english,
        &locale_set,
    );

    let file = syn::parse2(quote! {
        #locale_support
        #lookup
        #state_lookup
        #interpolated
    })
    .expect("generated code should parse");

    format!(
        "\
// Automatically generated by `cargo run -p accesskit_l10n_build`.
// Do not edit manually.

{}",
        prettyplease::unparse(&file)
    )
}

/// Read every `*.txt` translation file in `dir`, parse and validate them, and
/// return the generated Rust source together with any validation warnings.
pub fn generate_from_dir(dir: &Path) -> Result<(String, Vec<String>), String> {
    let mut paths = Vec::new();
    for entry in
        std::fs::read_dir(dir).map_err(|e| format!("failed to read {}: {e}", dir.display()))?
    {
        let entry = entry.map_err(|e| format!("failed to read directory entry: {e}"))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("txt") {
            paths.push(path);
        }
    }
    paths.sort();

    let mut files = Vec::new();
    for path in &paths {
        let locale = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("non-UTF-8 filename: {}", path.display()))?
            .to_lowercase();
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        let file = parse_translation_file(&locale, &content)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        files.push(file);
    }

    let result = validate(&files);
    if result.has_errors() {
        return Err(format!(
            "translation validation failed:\n{}",
            result.errors.join("\n")
        ));
    }

    Ok((generate_code(&files), result.warnings))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic() {
        let content = "role.banner = banner\nrole.main = main\n";
        let file = parse_translation_file("en", content).unwrap();
        assert_eq!(file.locale, "en");
        assert_eq!(file.entries["role.banner"], "banner");
        assert_eq!(file.entries["role.main"], "main");
    }

    #[test]
    fn parse_comments_and_empty_lines() {
        let content = "# This is a comment\n\nrole.banner = banner\n\n# Another comment\n";
        let file = parse_translation_file("en", content).unwrap();
        assert_eq!(file.entries.len(), 1);
        assert_eq!(file.entries["role.banner"], "banner");
    }

    #[test]
    fn parse_value_with_equals() {
        let content = "role.banner = a = b = c\n";
        let file = parse_translation_file("en", content).unwrap();
        assert_eq!(file.entries["role.banner"], "a = b = c");
    }

    #[test]
    fn parse_crlf_line_endings() {
        let content = "role.banner = banner\r\nrole.main = main\r\n";
        let file = parse_translation_file("en", content).unwrap();
        assert_eq!(file.entries.len(), 2);
        assert_eq!(file.entries["role.banner"], "banner");
    }

    #[test]
    fn parse_whitespace_trimming() {
        let content = "  role.banner  =  bannière  \n";
        let file = parse_translation_file("fr", content).unwrap();
        assert_eq!(file.entries["role.banner"], "bannière");
    }

    #[test]
    fn parse_unicode_values() {
        let content = "role.banner = バナー\n";
        let file = parse_translation_file("ja", content).unwrap();
        assert_eq!(file.entries["role.banner"], "バナー");
    }

    #[test]
    fn parse_placeholder_value() {
        let content = "heading_with_level = heading level {0}\n";
        let file = parse_translation_file("en", content).unwrap();
        assert_eq!(file.entries["heading_with_level"], "heading level {0}");
    }

    #[test]
    fn parse_error_missing_delimiter() {
        let content = "this has no equals sign\n";
        let err = parse_translation_file("en", content).unwrap_err();
        assert!(matches!(
            err,
            ParseError::MissingDelimiter { line_number: 1, .. }
        ));
    }

    #[test]
    fn parse_error_empty_key() {
        let content = " = some value\n";
        let err = parse_translation_file("en", content).unwrap_err();
        assert!(matches!(err, ParseError::EmptyKey { line_number: 1 }));
    }

    #[test]
    fn parse_error_duplicate_key() {
        let content = "role.banner = one\nrole.banner = two\n";
        let err = parse_translation_file("en", content).unwrap_err();
        assert!(matches!(
            err,
            ParseError::DuplicateKey { line_number: 2, .. }
        ));
    }

    #[test]
    fn parse_empty_file() {
        let content = "";
        let file = parse_translation_file("en", content).unwrap();
        assert!(file.entries.is_empty());
    }

    #[test]
    fn parse_only_comments() {
        let content = "# comment 1\n# comment 2\n";
        let file = parse_translation_file("en", content).unwrap();
        assert!(file.entries.is_empty());
    }

    #[test]
    fn parse_empty_value() {
        let content = "role.banner =\n";
        let file = parse_translation_file("en", content).unwrap();
        assert_eq!(file.entries["role.banner"], "");
    }

    #[test]
    fn extract_placeholders_basic() {
        let phs = extract_placeholders("heading level {0}");
        assert_eq!(phs, BTreeSet::from([0]));
    }

    #[test]
    fn extract_placeholders_multiple() {
        let phs = extract_placeholders("{0} of {1} selected");
        assert_eq!(phs, BTreeSet::from([0, 1]));
    }

    #[test]
    fn extract_placeholders_escaped_braces() {
        let phs = extract_placeholders("use {{0}} for literal braces");
        assert!(phs.is_empty());
    }

    #[test]
    fn extract_placeholders_none() {
        let phs = extract_placeholders("banner");
        assert!(phs.is_empty());
    }

    #[test]
    fn validate_missing_english() {
        let files = vec![TranslationFile {
            locale: "de".to_string(),
            entries: BTreeMap::from([("role.banner".to_string(), "Banner".to_string())]),
        }];
        let result = validate(&files);
        assert!(result.has_errors());
        assert!(result.errors[0].contains("missing required English"));
    }

    #[test]
    fn validate_ok() {
        let files = vec![
            TranslationFile {
                locale: "en".to_string(),
                entries: BTreeMap::from([("role.banner".to_string(), "banner".to_string())]),
            },
            TranslationFile {
                locale: "de".to_string(),
                entries: BTreeMap::from([("role.banner".to_string(), "Banner".to_string())]),
            },
        ];
        let result = validate(&files);
        assert!(!result.has_errors());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn validate_missing_key_in_non_english() {
        let files = vec![
            TranslationFile {
                locale: "en".to_string(),
                entries: BTreeMap::from([
                    ("role.banner".to_string(), "banner".to_string()),
                    ("role.main".to_string(), "main".to_string()),
                ]),
            },
            TranslationFile {
                locale: "de".to_string(),
                entries: BTreeMap::from([("role.banner".to_string(), "Banner".to_string())]),
            },
        ];
        let result = validate(&files);
        assert!(!result.has_errors());
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("missing key 'role.main'"));
    }

    #[test]
    fn validate_extra_key_in_non_english() {
        let files = vec![
            TranslationFile {
                locale: "en".to_string(),
                entries: BTreeMap::from([("role.banner".to_string(), "banner".to_string())]),
            },
            TranslationFile {
                locale: "de".to_string(),
                entries: BTreeMap::from([
                    ("role.banner".to_string(), "Banner".to_string()),
                    ("role.extra".to_string(), "extra".to_string()),
                ]),
            },
        ];
        let result = validate(&files);
        assert!(result.has_errors());
        assert!(result.errors[0].contains("not found in English"));
    }

    #[test]
    fn validate_placeholder_mismatch() {
        let files = vec![
            TranslationFile {
                locale: "en".to_string(),
                entries: BTreeMap::from([(
                    "heading_with_level".to_string(),
                    "heading level {0}".to_string(),
                )]),
            },
            TranslationFile {
                locale: "de".to_string(),
                entries: BTreeMap::from([(
                    "heading_with_level".to_string(),
                    "Überschrift Ebene {0} {1}".to_string(),
                )]),
            },
        ];
        let result = validate(&files);
        assert!(result.has_errors());
        assert!(result.errors[0].contains("placeholders"));
    }

    #[test]
    fn validate_placeholder_match() {
        let files = vec![
            TranslationFile {
                locale: "en".to_string(),
                entries: BTreeMap::from([(
                    "heading_with_level".to_string(),
                    "heading level {0}".to_string(),
                )]),
            },
            TranslationFile {
                locale: "de".to_string(),
                entries: BTreeMap::from([(
                    "heading_with_level".to_string(),
                    "Überschrift Ebene {0}".to_string(),
                )]),
            },
        ];
        let result = validate(&files);
        assert!(!result.has_errors());
    }

    #[test]
    fn generate_code_basic() {
        let files = vec![TranslationFile {
            locale: "en".to_string(),
            entries: BTreeMap::from([
                ("role.banner".to_string(), "banner".to_string()),
                (
                    "heading_with_level".to_string(),
                    "heading level {0}".to_string(),
                ),
            ]),
        }];
        let code = generate_code(&files);
        // Static role -> lookup_impl.
        assert!(code.contains("lookup_impl"));
        assert!(code.contains("Role::Banner"));
        assert!(code.contains("\"banner\""));
        // Templated role -> InterpolatedString variant + writer, with the
        // positional placeholder rewritten to a captured name.
        assert!(code.contains("enum InterpolatedString"));
        assert!(code.contains("HeadingWithLevel(&'a str)"));
        assert!(code.contains("write_interpolated_string_impl"));
        assert!(code.contains("heading level {a0}"));
    }

    #[test]
    fn generate_code_multiple_locales() {
        let files = vec![
            TranslationFile {
                locale: "en".to_string(),
                entries: BTreeMap::from([("role.banner".to_string(), "banner".to_string())]),
            },
            TranslationFile {
                locale: "de".to_string(),
                entries: BTreeMap::from([("role.banner".to_string(), "Banner".to_string())]),
            },
        ];
        let code = generate_code(&files);
        // Locales are sorted, so "de" is index 0 and "en" is index 1. The arms
        // match on the generated per-locale constant, not the locale string.
        assert!(code.contains("static LOCALE_TAGS"));
        assert!(code.contains("const DE: usize = 0;"));
        assert!(code.contains("const EN: usize = 1;"));
        assert!(code.contains("(DE, Role::Banner) => Some(\"Banner\")"));
        assert!(code.contains("(EN, Role::Banner) => Some(\"banner\")"));
    }

    #[test]
    fn generate_code_bakes_english_fallback() {
        let files = vec![
            TranslationFile {
                locale: "en".to_string(),
                entries: BTreeMap::from([
                    ("role.banner".to_string(), "banner".to_string()),
                    ("role.main".to_string(), "main".to_string()),
                ]),
            },
            TranslationFile {
                // Missing role.main: its row should be filled with English.
                locale: "de".to_string(),
                entries: BTreeMap::from([("role.banner".to_string(), "Banner".to_string())]),
            },
        ];
        let code = generate_code(&files);
        // de is index 0: its own banner, English-filled main.
        assert!(code.contains("(DE, Role::Banner) => Some(\"Banner\")"));
        assert!(code.contains("(DE, Role::Main) => Some(\"main\")"));
    }

    #[test]
    fn generate_code_bakes_base_language_fallback() {
        let files = vec![
            TranslationFile {
                locale: "en".to_string(),
                entries: BTreeMap::from([("role.banner".to_string(), "banner".to_string())]),
            },
            TranslationFile {
                locale: "es".to_string(),
                entries: BTreeMap::from([("role.banner".to_string(), "estandarte".to_string())]),
            },
            TranslationFile {
                // Empty: falls back to its base language "es", not English.
                locale: "es-419".to_string(),
                entries: BTreeMap::new(),
            },
        ];
        let code = generate_code(&files);
        // Sorted: en=0, es=1, es-419=2. es-419 inherits "es", not English.
        // Its hyphen becomes an underscore in the generated constant name.
        assert!(code.contains("const ES_419: usize = 2;"));
        assert!(code.contains("(ES_419, Role::Banner) => Some(\"estandarte\")"));
    }

    #[test]
    fn generate_code_escapes_special_characters() {
        let files = vec![TranslationFile {
            locale: "en".to_string(),
            entries: BTreeMap::from([(
                "role.banner".to_string(),
                "a \"quoted\" back\\slash".to_string(),
            )]),
        }];
        let code = generate_code(&files);
        assert!(code.contains(r#""a \"quoted\" back\\slash""#));
    }

    #[test]
    fn generate_code_preserves_non_ascii_literally() {
        // Non-ASCII values must appear verbatim, not as `\u{...}` escapes, so
        // the committed generated file stays readable.
        let files = vec![
            TranslationFile {
                locale: "en".to_string(),
                entries: BTreeMap::from([("role.banner".to_string(), "banner".to_string())]),
            },
            TranslationFile {
                locale: "ar".to_string(),
                entries: BTreeMap::from([("role.banner".to_string(), "مربّع حوار".to_string())]),
            },
        ];
        let code = generate_code(&files);
        assert!(code.contains("Some(\"مربّع حوار\")"));
        assert!(!code.contains("\\u{"));
    }

    #[test]
    fn generate_code_output_parses_as_rust() {
        let files = vec![
            TranslationFile {
                locale: "en".to_string(),
                entries: BTreeMap::from([
                    ("role.banner".to_string(), "banner".to_string()),
                    (
                        "heading_with_level".to_string(),
                        "heading level {0}".to_string(),
                    ),
                    ("state.toggle_button_on".to_string(), "On".to_string()),
                    (
                        "state.multiselectable".to_string(),
                        "multiselectable, {0} of {1} selected.".to_string(),
                    ),
                ]),
            },
            TranslationFile {
                locale: "en-gb".to_string(),
                entries: BTreeMap::from([("role.banner".to_string(), "banner".to_string())]),
            },
        ];
        let code = generate_code(&files);
        syn::parse_file(&code).expect("generated code should parse as Rust");
    }

    #[test]
    fn generate_code_is_deterministic() {
        let files = vec![
            TranslationFile {
                locale: "de".to_string(),
                entries: BTreeMap::from([("role.banner".to_string(), "Banner".to_string())]),
            },
            TranslationFile {
                locale: "en".to_string(),
                entries: BTreeMap::from([("role.banner".to_string(), "banner".to_string())]),
            },
        ];
        let reversed: Vec<_> = files.iter().rev().cloned().collect();
        assert_eq!(generate_code(&files), generate_code(&reversed));
    }

    #[test]
    fn generate_code_unknown_key_ignored() {
        let files = vec![TranslationFile {
            locale: "en".to_string(),
            entries: BTreeMap::from([("unknown.key".to_string(), "value".to_string())]),
        }];
        let code = generate_code(&files);
        // Should not contain the unknown key in any match arm
        assert!(!code.contains("unknown"));
    }

    #[test]
    fn parse_error_display() {
        let err = ParseError::MissingDelimiter {
            line_number: 5,
            line: "no equals here".to_string(),
        };
        let s = format!("{err}");
        assert!(s.contains("line 5"));
        assert!(s.contains("no equals here"));
    }

    #[test]
    fn key_to_state_variant_basic() {
        assert_eq!(
            key_to_state_variant("state.toggle_button_on"),
            Some("ToggleButtonOn".to_string())
        );
        assert_eq!(
            key_to_state_variant("state.aria_current_page"),
            Some("AriaCurrentPage".to_string())
        );
        assert_eq!(
            key_to_state_variant("state.invalid_true"),
            Some("InvalidTrue".to_string())
        );
        assert_eq!(
            key_to_state_variant("state.radio_checked"),
            Some("RadioChecked".to_string())
        );
    }

    #[test]
    fn key_to_state_variant_not_state_prefix() {
        assert_eq!(key_to_state_variant("role.banner"), None);
        assert_eq!(key_to_state_variant("heading_with_level"), None);
        assert_eq!(key_to_state_variant("unknown.key"), None);
    }

    #[test]
    fn generate_code_with_state_entries() {
        let files = vec![TranslationFile {
            locale: "en".to_string(),
            entries: BTreeMap::from([
                ("state.toggle_button_on".to_string(), "On".to_string()),
                (
                    "state.multiselectable".to_string(),
                    "multiselectable, {0} of {1} selected.".to_string(),
                ),
                ("role.banner".to_string(), "banner".to_string()),
            ]),
        }];
        let code = generate_code(&files);
        // Role entries still work
        assert!(code.contains("Role::Banner"));
        // Static state entry -> state_lookup_impl
        assert!(code.contains("StateKey::ToggleButtonOn"));
        assert!(code.contains("state_lookup_impl"));
        // Templated state entry -> InterpolatedString variant (two args)
        assert!(code.contains("Multiselectable(&'a str, &'a str)"));
        assert!(code.contains("multiselectable, {a0} of {a1} selected."));
    }

    #[test]
    fn validate_empty_english() {
        let files = vec![TranslationFile {
            locale: "en".to_string(),
            entries: BTreeMap::new(),
        }];
        let result = validate(&files);
        assert!(result.has_errors());
        assert!(result.errors[0].contains("empty"));
    }
}
