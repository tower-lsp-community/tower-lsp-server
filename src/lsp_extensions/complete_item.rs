//! extension for [lsp_types::CompletionItem]
//
//

use lsp_types::CompletionItem;
/// extension for [lsp_types::CompletionItem]
pub trait CompletionItemEx {
    /// Simple CompletionItem
    fn new_simple<'a>(label: &'a str, detail: impl Into<Option<&'a str>>) -> Self;
}

impl CompletionItemEx for CompletionItem {
    fn new_simple<'a>(label: &'a str, detail: impl Into<Option<&'a str>>) -> Self {
        let detail = detail.into();
        let detail = detail.map(|d| d.to_owned());
        Self {
            label: label.to_owned(),
            detail,
            ..Default::default()
        }
    }
}
