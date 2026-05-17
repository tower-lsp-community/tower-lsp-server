//! extension for [lsp_types::ChangeNotifications] and etc
//
//

use lsp_types::ChangeNotifications;

/// extension for [lsp_types::ChangeNotifications]
pub trait ChangeNotificationsEx {
    /// create a new [lsp_types::ChangeNotifications]
    fn new_some(change_notifications: impl Into<Self>) -> Option<Self>
    where
        Self: Sized;
}

impl ChangeNotificationsEx for ChangeNotifications {
    fn new_some(change_notifications: impl Into<Self>) -> Option<Self>
    where
        Self: Sized,
    {
        Some(change_notifications.into())
    }
}
