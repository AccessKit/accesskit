// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{Role, Toggled};
use accesskit_consumer::{FilterResult, Node, TreeState};
use web_sys::HtmlElement;

use crate::filters::{filter, filter_with_root_exception};

pub(crate) struct NodeWrapper<'a>(pub(crate) Node<'a>);

impl<'a> NodeWrapper<'a> {
    fn role(&self) -> Option<String> {
        let role = self.0.role();
        match role {
            Role::Cell => Some("cell".into()),
            Role::Image => Some("img".into()),
            Role::Link => Some("link".into()),
            Role::Row => Some("row".into()),
            Role::ListItem => Some("listitem".into()),
            Role::TreeItem => Some("treeitem".into()),
            Role::ListBoxOption => Some("option".into()),
            Role::MenuItem | Role::MenuListOption => Some("menuitem".into()),
            Role::Paragraph => Some("paragraph".into()),
            Role::GenericContainer => Some("generic".into()),
            Role::CheckBox => Some("checkbox".into()),
            Role::RadioButton => Some("radio".into()),
            Role::TextInput => Some("textbox".into()),
            Role::Button | Role::DefaultButton => Some("button".into()),
            Role::RowHeader => Some("rowheader".into()),
            Role::ColumnHeader => Some("columnheader".into()),
            Role::RowGroup => Some("rowgroup".into()),
            Role::List => Some("list".into()),
            Role::Table => Some("table".into()),
            Role::Switch => Some("switch".into()),
            Role::Menu => Some("menu".into()),
            Role::MultilineTextInput => Some("textbox".into()),
            Role::SearchInput => Some("searchbox".into()),
            Role::DateInput
            | Role::DateTimeInput
            | Role::WeekInput
            | Role::MonthInput
            | Role::TimeInput
            | Role::EmailInput
            | Role::NumberInput
            | Role::PasswordInput
            | Role::PhoneNumberInput
            | Role::UrlInput => Some("textbox".into()),
            Role::Alert => Some("alert".into()),
            Role::AlertDialog => Some("alertdialog".into()),
            Role::Application => Some("application".into()),
            Role::Article => Some("article".into()),
            Role::Banner => Some("banner".into()),
            Role::Blockquote => Some("blockquote".into()),
            Role::Caption => Some("caption".into()),
            Role::Code => Some("code".into()),
            Role::ComboBox | Role::EditableComboBox => Some("combobox".into()),
            Role::Complementary => Some("complementary".into()),
            Role::Comment => Some("comment".into()),
            Role::ContentDeletion => Some("deletion".into()),
            Role::ContentInsertion => Some("insertion".into()),
            Role::ContentInfo => Some("contentinfo".into()),
            Role::Definition => Some("definition".into()),
            Role::Dialog => Some("dialog".into()),
            Role::Directory => Some("directory".into()),
            Role::Document => Some("document".into()),
            Role::Emphasis => Some("emphasis".into()),
            Role::Feed => Some("feed".into()),
            Role::Figure => Some("figure".into()),
            Role::Footer => Some("contentinfo".into()),
            Role::Form => Some("form".into()),
            Role::Grid => Some("grid".into()),
            Role::Group => Some("group".into()),
            Role::Header => Some("banner".into()),
            Role::Heading => Some("heading".into()),
            Role::ListBox => Some("listbox".into()),
            Role::Log => Some("log".into()),
            Role::Main => Some("main".into()),
            Role::Mark => Some("mark".into()),
            Role::Marquee => Some("marquee".into()),
            Role::Math => Some("math".into()),
            Role::MenuBar => Some("menubar".into()),
            Role::MenuItemCheckBox => Some("menuitemcheckbox".into()),
            Role::MenuItemRadio => Some("menuitemradio".into()),
            Role::Meter => Some("meter".into()),
            Role::Navigation => Some("navigation".into()),
            Role::Note => Some("note".into()),
            Role::ProgressIndicator => Some("progressbar".into()),
            Role::RadioGroup => Some("radiogroup".into()),
            Role::Region => Some("region".into()),
            Role::ScrollBar => Some("scrollbar".into()),
            Role::Search => Some("search".into()),
            Role::Section => Some("section".into()),
            Role::Slider => Some("slider".into()),
            Role::SpinButton => Some("spinbutton".into()),
            Role::Splitter => Some("separator".into()),
            Role::Status => Some("status".into()),
            Role::Strong => Some("strong".into()),
            Role::Suggestion => Some("suggestion".into()),
            Role::Tab => Some("tab".into()),
            Role::TabList => Some("tablist".into()),
            Role::TabPanel => Some("tabpanel".into()),
            Role::Term => Some("term".into()),
            Role::Time => Some("time".into()),
            Role::Timer => Some("timer".into()),
            Role::Toolbar => Some("toolbar".into()),
            Role::Tooltip => Some("tooltip".into()),
            Role::Tree => Some("tree".into()),
            Role::TreeGrid => Some("treegrid".into()),
            Role::Window => Some("window".into()),
            Role::GraphicsDocument => Some("graphics-document".into()),
            Role::GraphicsObject => Some("graphics-object".into()),
            Role::GraphicsSymbol => Some("graphics-symbol".into()),
            Role::DocAbstract => Some("doc-abstract".into()),
            Role::DocAcknowledgements => Some("doc-acknowledgements".into()),
            Role::DocAfterword => Some("doc-afterword".into()),
            Role::DocAppendix => Some("doc-appendix".into()),
            Role::DocBackLink => Some("doc-backlink".into()),
            Role::DocBiblioEntry => Some("doc-biblioentry".into()),
            Role::DocBibliography => Some("doc-bibliography".into()),
            Role::DocBiblioRef => Some("doc-biblioref".into()),
            Role::DocChapter => Some("doc-chapter".into()),
            Role::DocColophon => Some("doc-colophon".into()),
            Role::DocConclusion => Some("doc-conclusion".into()),
            Role::DocCover => Some("doc-cover".into()),
            Role::DocCredit => Some("doc-credit".into()),
            Role::DocCredits => Some("doc-credits".into()),
            Role::DocDedication => Some("doc-dedication".into()),
            Role::DocEndnote => Some("doc-endnote".into()),
            Role::DocEndnotes => Some("doc-endnotes".into()),
            Role::DocEpigraph => Some("doc-epigraph".into()),
            Role::DocEpilogue => Some("doc-epilogue".into()),
            Role::DocErrata => Some("doc-errata".into()),
            Role::DocExample => Some("doc-example".into()),
            Role::DocFootnote => Some("doc-footnote".into()),
            Role::DocForeword => Some("doc-forward".into()),
            Role::DocGlossary => Some("doc-glossary".into()),
            Role::DocGlossRef => Some("doc-glossref".into()),
            Role::DocIndex => Some("doc-index".into()),
            Role::DocIntroduction => Some("doc-introduction".into()),
            Role::DocNoteRef => Some("doc-noteref".into()),
            Role::DocNotice => Some("doc-notice".into()),
            Role::DocPageBreak => Some("doc-pagebreak".into()),
            Role::DocPageFooter => Some("doc-pagefooter".into()),
            Role::DocPageHeader => Some("doc-pageheader".into()),
            Role::DocPageList => Some("doc-pagelist".into()),
            Role::DocPart => Some("doc-part".into()),
            Role::DocPreface => Some("doc-preface".into()),
            Role::DocPrologue => Some("doc-prologue".into()),
            Role::DocPullquote => Some("doc-pullquote".into()),
            Role::DocQna => Some("doc-qna".into()),
            Role::DocSubtitle => Some("doc-subtitle".into()),
            Role::DocTip => Some("doc-tip".into()),
            Role::DocToc => Some("doc-toc".into()),
            _ => None,
        }
    }

    fn tabindex(&self) -> Option<String> {
        self.0.is_focusable().then(|| "-1".into())
    }

    fn name(&self) -> Option<String> {
        self.0.name()
    }

    fn aria_label(&self) -> Option<String> {
        if self.0.role() == Role::Label {
            return None;
        }
        self.name()
    }

    fn text_content(&self) -> Option<String> {
        if self.0.role() != Role::Label {
            return None;
        }
        self.name()
    }

    fn aria_checked(&self) -> Option<String> {
        self.0.toggled().map(|value| match value {
            Toggled::False => "false".into(),
            Toggled::True => "true".into(),
            Toggled::Mixed => "mixed".into(),
        })
    }

    fn aria_valuemax(&self) -> Option<String> {
        self.0.max_numeric_value().map(|value| value.to_string())
    }

    fn aria_valuemin(&self) -> Option<String> {
        self.0.min_numeric_value().map(|value| value.to_string())
    }

    fn aria_valuenow(&self) -> Option<String> {
        self.0.numeric_value().map(|value| value.to_string())
    }

    fn aria_valuetext(&self) -> Option<String> {
        self.0.value()
    }
}

macro_rules! attributes {
    ($(($name:literal, $m:ident)),+) => {
        impl NodeWrapper<'_> {
            pub(crate) fn set_all_attributes(&self, element: &HtmlElement) {
                $(let value = self.$m();
                if let Some(value) = value.as_ref() {
                    element.set_attribute(&$name, value).unwrap();
                })*
                if let Some(text_content) = self.text_content().as_ref() {
                    element.set_text_content(Some(text_content));
                }
            }
            pub(crate) fn update_attributes(&self, element: &HtmlElement, old: &NodeWrapper) {
                $({
                    let old_value = old.$m();
                    let new_value = self.$m();
                    if old_value != new_value {
                        if let Some(value) = new_value.as_ref() {
                            element.set_attribute(&$name, value).unwrap();
                        } else {
                            element.remove_attribute(&$name).unwrap();
                        }
                    }
                })*
                let old_text_content = old.text_content();
                let new_text_content = self.text_content();
                if old_text_content != new_text_content {
                    if let Some(text_content) = new_text_content.as_ref() {
                        element.set_text_content(Some(text_content));
                    } else {
                        element.set_text_content(None);
                    }
                }
            }
        }
    };
}

attributes! {
    ("role", role),
    ("tabindex", tabindex),
    ("aria-label", aria_label),
    ("aria-checked", aria_checked),
    ("aria-valuemax", aria_valuemax),
    ("aria-valuemin", aria_valuemin),
    ("aria-valuenow", aria_valuenow),
    ("aria-valuetext", aria_valuetext)
}
