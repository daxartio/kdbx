use log::*;

use crate::Result;

#[cfg(feature = "clipboard")]
use arboard::Clipboard;

#[cfg(feature = "clipboard")]
pub fn set_clipboard(val: Option<String>) -> Result<()> {
    Clipboard::new()
        .and_then(|mut clipboard| clipboard.set_text(val.unwrap_or_default()))
        .map_err(|e| {
            warn!("could not set the clipboard: {}", e);
            e.into()
        })
}

#[cfg(not(feature = "clipboard"))]
pub fn set_clipboard(_: Option<String>) -> Result<()> {
    Err("Feature clipboard is not available.".into())
}
