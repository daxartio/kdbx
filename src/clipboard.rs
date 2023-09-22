use crate::Result;

#[cfg(feature = "clipboard")]
use clipboard::{ClipboardContext, ClipboardProvider};

use log::*;

#[cfg(feature = "clipboard")]
pub fn set_clipboard(val: Option<String>) -> Result<()> {
    ClipboardProvider::new()
        .and_then(|mut ctx: ClipboardContext| ctx.set_contents(val.unwrap_or_default()))
        .map_err(|e| {
            warn!("could not set the clipboard: {}", e);
            e
        })
}

#[cfg(not(feature = "clipboard"))]
pub fn set_clipboard(_: Option<String>) -> Result<()> {
    Err("Feature clipboard is not available.".into())
}
