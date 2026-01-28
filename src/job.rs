use nix::{
    sys::signal::{kill, Signal},
    unistd::{tcsetpgrp, Pid},
};
use std::process::Child;

use crate::{
    prompt::{prompt_user, PromptAction},
    terminal::{reset_terminal, TerminalState},
    ui::update_prompt_with_guide,
};

pub fn give_terminal(fd: i32, pgrp: Pid) {
    unsafe {
        libc::signal(libc::SIGTTOU, libc::SIG_IGN);
    }
    let _ = tcsetpgrp(fd, pgrp);
}

fn stop(pid: Pid) {
    let _ = kill(pid, Signal::SIGSTOP);
}

fn cont(pid: Pid) {
    let _ = kill(pid, Signal::SIGCONT);
}

pub fn suspend_and_prompt(child: &mut Child, child_pid: Pid, parent_pgrp: Pid, stdin_fd: i32) {
    let term = TerminalState::capture(stdin_fd);

    stop(child_pid);
    give_terminal(stdin_fd, parent_pgrp);
    term.set_cooked();

    match prompt_user() {
        PromptAction::Close => {
            let _ = child.kill();
            reset_terminal(stdin_fd, parent_pgrp);
            std::process::exit(124);
        }
        PromptAction::Resume => {
            update_prompt_with_guide();
            term.restore();
            give_terminal(stdin_fd, child_pid);
            cont(child_pid);
        }
    }
}
