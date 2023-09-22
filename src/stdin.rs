use crate::pwd::Pwd;

use libc::{isatty, tcgetattr, tcsetattr, ECHO, ECHONL, STDIN_FILENO, TCSANOW};

use log::*;

use std::io::{self, Read};
use std::mem::MaybeUninit;

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

fn reset_impl(termios: Option<libc::termios>) {
    if let Some(termios) = termios {
        unsafe { tcsetattr(STDIN_FILENO, TCSANOW, &termios) };
    }
}

fn trim_newlines(password: &mut String) {
    while password.ends_with(['\n', '\r'].as_ref()) {
        password.pop();
    }
}
