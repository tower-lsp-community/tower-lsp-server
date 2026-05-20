//! extension for [lsp_types::TextDocumentContentChangeEvent] and etc
//
//

use lsp_types::{TextDocumentContentChangeEvent, TextDocumentContentChangePartial};

/// Extension for TextDocumentContentChangeEvent
pub trait TextDocumentContentChangeEventEx {
    /// Get whole_content
    fn whole_content(&self) -> Option<&str>;
    /// Get partial_content
    fn partial_content(&self) -> Option<&TextDocumentContentChangePartial>;
}

impl TextDocumentContentChangeEventEx for TextDocumentContentChangeEvent {
    fn whole_content(&self) -> Option<&str> {
        match self {
            Self::TextDocumentContentChangeWholeDocument(context) => Some(&context.text),
            Self::TextDocumentContentChangePartial(_) => None,
        }
    }
    fn partial_content(&self) -> Option<&TextDocumentContentChangePartial> {
        match self {
            Self::TextDocumentContentChangeWholeDocument(_) => None,
            Self::TextDocumentContentChangePartial(partial_content) => Some(partial_content),
        }
    }
}
