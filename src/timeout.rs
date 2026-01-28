use std::{
    io,
    os::fd::AsRawFd,
    process::Command,
    time::{Duration, Instant},
};

use nix::unistd::{getpgrp, setpgid, Pid};

use crate::{job::suspend_and_prompt, signal::install_sigwinch};

pub fn run(mut args: Vec<String>) {
    if args.len() < 2 {
        eprintln!("usage: toomuch <duration> <command> [args...]");
        std::process::exit(1);
    }

    install_sigwinch();

    let timeout = parse_duration(&args.remove(0)).unwrap_or_else(|e| {
        eprintln!("invalid timeout: {}", e);
        std::process::exit(1);
    });

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

fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();

    if s.is_empty() {
        return Err("empty duration".into());
    }

    // Split numeric part and unit
    let (num, unit) = s
        .chars()
        .position(|c| !c.is_ascii_digit())
        .map(|i| s.split_at(i))
        .unwrap_or((s, "s")); // default to seconds if no unit

    let value: u64 = num
        .parse()
        .map_err(|_| format!("invalid number in duration: {}", s))?;

    match unit {
        "s" => Ok(Duration::from_secs(value)),
        "m" => Ok(Duration::from_secs(value * 60)),
        "h" => Ok(Duration::from_secs(value * 60 * 60)),
        "ms" => Ok(Duration::from_millis(value)),
        _ => Err(format!("unknown duration unit: {}", unit)),
    }
}
