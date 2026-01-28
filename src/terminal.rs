use nix::unistd::Pid;
use termios::{tcsetattr, Termios, ECHO, ICANON, TCSADRAIN};

pub struct TerminalState {
    fd: i32,
    saved: Option<Termios>,
}

impl TerminalState {
    pub fn capture(fd: i32) -> Self {
        Self {
            fd,
            saved: Termios::from_fd(fd).ok(),
        }
    }

    pub fn set_cooked(&self) {
        if let Ok(mut t) = Termios::from_fd(self.fd) {
            t.c_lflag |= ICANON | ECHO;
            t.c_iflag |= termios::ICRNL;
            let _ = tcsetattr(self.fd, TCSADRAIN, &t);
        }
    }

    pub fn restore(&self) {
        if let Some(ref t) = self.saved {
            let _ = tcsetattr(self.fd, TCSADRAIN, t);
        }
    }
}

pub fn reset_terminal(fd: i32, parent_pgrp: Pid) {
    crate::job::give_terminal(fd, parent_pgrp);

    if let Ok(mut t) = Termios::from_fd(fd) {
        t.c_lflag |= ICANON | ECHO;
        t.c_iflag |= termios::ICRNL;
        t.c_oflag |= termios::ONLCR;
        let _ = tcsetattr(fd, termios::TCSANOW, &t);
    }
}
