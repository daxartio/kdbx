use log::*;

use crate::{pwd::Pwd, Result};

#[cfg(feature = "clipboard")]
use arboard::Clipboard;

#[cfg(feature = "clipboard")]
pub fn set_clipboard(val: Option<Pwd>) -> Result<()> {
    Clipboard::new()
        .and_then(|mut clipboard| clipboard.set_text(val.as_deref().unwrap_or_default()))
        .map_err(|e| {
            warn!("could not set the clipboard: {}", e);
            e.into()
        })
}

#[cfg(not(feature = "clipboard"))]
pub fn set_clipboard(_: Option<Pwd>) -> Result<()> {
    Err("Feature clipboard is not available.".into())
}
