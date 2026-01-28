use std::{
    io,
    os::fd::AsRawFd,
    process::Command,
    time::{Duration, Instant},
};

use nix::unistd::{Pid, getpgrp, setpgid};

use crate::{job::suspend_and_prompt, signal::install_sigwinch};

pub fn run(mut args: Vec<String>) {
    if args.len() < 2 {
        eprintln!("usage: toomuch <seconds> <command> [args...]");
        std::process::exit(1);
    }

    install_sigwinch();

    let timeout = Duration::from_secs(args.remove(0).parse().unwrap());
    let cmd = args.remove(0);

    let stdin_fd = io::stdin().as_raw_fd();
    let parent_pgrp = getpgrp();

    let mut child = Command::new(cmd).args(args).spawn().expect("spawn failed");

    let child_pid = Pid::from_raw(child.id() as i32);
    let _ = setpgid(child_pid, child_pid);

    crate::job::give_terminal(stdin_fd, child_pid);

    let start = Instant::now();
    let mut prompted = false;

    loop {
        if let Ok(Some(status)) = child.try_wait() {
            crate::job::give_terminal(stdin_fd, parent_pgrp);
            std::process::exit(status.code().unwrap_or(0));
        }

        if !prompted && start.elapsed() >= timeout {
            prompted = true;
            suspend_and_prompt(&mut child, child_pid, parent_pgrp, stdin_fd);
        }
    }
}
