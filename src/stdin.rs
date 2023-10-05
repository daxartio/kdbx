use std::{
    io::{self, Read},
    mem::MaybeUninit,
};

use libc::{isatty, tcgetattr, tcsetattr, ECHO, ECHONL, STDIN_FILENO, TCSANOW};
use log::*;

use crate::pwd::Pwd;

pub struct Stdin(Option<libc::termios>);

impl Drop for Stdin {
    fn drop(&mut self) {
        self.reset_tty();
    }
}

impl Stdin {
    pub fn new() -> Self {
        new_impl().unwrap_or_else(|e| {
            warn!("platform API call error: {}", e);
            Stdin(None)
        })
    }

    pub fn read_password(&self) -> Pwd {
        let pwd = read_password(self.0).unwrap().into();
        self.reset_tty();
        pwd
    }

    pub fn read_text(&self) -> String {
        let text = read_text(self.0).unwrap();
        self.reset_tty();
        text
    }

    pub fn reset_tty(&self) {
        info!("resetting TTY params");
        reset_impl(self.0);
    }
}

fn new_impl() -> ::std::io::Result<Stdin> {
    unsafe {
        let mut termios = MaybeUninit::uninit();

        if isatty(STDIN_FILENO) != 1 {
            return Err(io::Error::new(io::ErrorKind::Other, "stdin is not a tty"));
        }

        if tcgetattr(STDIN_FILENO, termios.as_mut_ptr()) == 0 {
            return Ok(Stdin(Some(termios.assume_init())));
        }
    }

    Err(io::Error::last_os_error())
}

fn read_password(tty: Option<libc::termios>) -> ::std::io::Result<String> {
    let mut password = String::new();

    if let Some(mut termios) = tty {
        info!("read_password() :: TTY");

        termios.c_lflag &= !ECHO;
        termios.c_lflag |= ECHONL;

        unsafe { tcsetattr(STDIN_FILENO, TCSANOW, &termios) };

        io::stdin().read_line(&mut password)?;
    } else {
        info!("read_password() :: NOT A TTY");
        io::stdin().read_to_string(&mut password)?;
    }

    trim_newlines(&mut password);

    Ok(password)
}

fn read_text(tty: Option<libc::termios>) -> ::std::io::Result<String> {
    let mut text = String::new();

    if let Some(termios) = tty {
        info!("read_text() :: TTY");

        unsafe { tcsetattr(STDIN_FILENO, TCSANOW, &termios) };

        io::stdin().read_line(&mut text)?;
    } else {
        info!("read_text() :: NOT A TTY");
        io::stdin().read_to_string(&mut text)?;
    }

    trim_newlines(&mut text);

    Ok(text)
}

fn reset_impl(termios: Option<libc::termios>) {
    if let Some(termios) = termios {
        unsafe { tcsetattr(STDIN_FILENO, TCSANOW, &termios) };
    }
}

fn trim_newlines(text: &mut String) {
    while text.ends_with(['\n', '\r'].as_ref()) {
        text.pop();
    }
}
