use log::*;

use std::{convert::AsRef, ops::Deref, ptr, sync::atomic};

pub struct Pwd(String);

impl From<String> for Pwd {
    fn from(pwd: String) -> Self {
        Pwd(pwd)
    }
}

impl Deref for Pwd {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl AsRef<str> for Pwd {
    fn as_ref(&self) -> &str {
        self
    }
}

impl Drop for Pwd {
    fn drop(&mut self) {
        info!("zeroing password memory");
        zero_memory(&mut self.0)
    }
}

fn zero_memory(pwd: &mut String) {
    unsafe {
        for byte in pwd.as_bytes_mut() {
            ptr::write_volatile(byte, 0x00);
        }
    }

    atomic::fence(atomic::Ordering::SeqCst);
    atomic::compiler_fence(atomic::Ordering::SeqCst);
}
