use std::{convert::AsRef, ops::Deref};

use zeroize::Zeroizing;

#[derive(Clone)]
pub struct Pwd(Zeroizing<String>);

impl Default for Pwd {
    fn default() -> Self {
        Self(String::default().into())
    }
}

impl From<String> for Pwd {
    fn from(pwd: String) -> Self {
        Self(pwd.into())
    }
}

impl Deref for Pwd {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl AsRef<str> for Pwd {
    fn as_ref(&self) -> &str {
        self
    }
}

impl PartialEq for Pwd {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}
