//! lsp extension traits for gen-lsp-types
//!
pub mod complete_item;

pub use complete_item::*;

pub mod notifications;

pub use notifications::*;

pub mod text_document_content;
pub use text_document_content::*;
