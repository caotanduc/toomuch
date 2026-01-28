use nix::{
    sys::signal::{kill, Signal},
    unistd::{getpgrp, setpgid, tcsetpgrp, Pid},
};
use std::{
    env,
    io::{self, Write},
    os::fd::AsRawFd,
    process::{Child, Command},
    time::{Duration, Instant},
};
use termios::{tcsetattr, Termios, ICANON, ECHO, TCSADRAIN};
use terminal_size::{Width, Height, terminal_size};

fn draw_centered_prompt() {
    let size = terminal_size();
    if let Some((Width(w), Height(h))) = size {
        let term_width = w as usize;
        let term_height = h as usize;

        // Box dimensions
        let box_width = 60;
        let box_height = 5;

        // Calculate top-left position to center the box
        let start_row = (term_height / 2).saturating_sub(box_height / 2);
        let start_col = (term_width / 2).saturating_sub(box_width / 2);

        // Clear screen
        print!("\x1b[2J");

        // Draw top border
        print!("\x1b[{};{}H", start_row, start_col);
        print!("┌{}┐", "─".repeat(box_width - 2));

        // Draw middle lines
        for i in 1..box_height - 1 {
            print!("\x1b[{};{}H", start_row + i, start_col);
            print!("│{}│", " ".repeat(box_width - 2));
        }

        // Draw bottom border
        print!("\x1b[{};{}H", start_row + box_height - 1, start_col);
        print!("└{}┘", "─".repeat(box_width - 2));

        // Draw the prompt text centered in the box
        let prompt_text = "[toomuch] Time limit exceeded.";
        let options_text = "(c) close | (r) resume";

        let text_row = start_row + 1;
        let options_row = start_row + 2;
        let input_row = start_row + 3;

        // Center the title text
        let text_col = start_col + (box_width / 2).saturating_sub(prompt_text.len() / 2);
        print!("\x1b[{};{}H", text_row, text_col);
        print!("\x1b[31;1m{}\x1b[0m", prompt_text);

        // Center the options text
        let options_col = start_col + (box_width / 2).saturating_sub(options_text.len() / 2);
        print!("\x1b[{};{}H", options_row, options_col);
        print!("{}", options_text);

        // Position cursor for input
        let input_prompt = "> ";
        let input_col = start_col + (box_width / 2).saturating_sub(input_prompt.len() / 2);
        print!("\x1b[{};{}H", input_row, input_col);
        print!("{}", input_prompt);

        io::stdout().flush().unwrap();
    } else {
        // Fallback if terminal size cannot be determined
        print!("\x1b[2J\x1b[H");
        print!("\x1b[31;1m[toomuch]\x1b[0m Time limit exceeded. (c) close | (r) resume > ");
        io::stdout().flush().unwrap();
    }
}

fn update_prompt_with_guide() {
    let size = terminal_size();
    if let Some((Width(w), Height(h))) = size {
        let term_width = w as usize;
        let term_height = h as usize;

        // Box dimensions (same as prompt)
        let box_width = 60;
        let box_height = 5;

        // Calculate position (same as prompt for in-place update)
        let start_row = (term_height / 2).saturating_sub(box_height / 2);
        let start_col = (term_width / 2).saturating_sub(box_width / 2);

        // Clear the interior of the box (don't redraw borders)
        for i in 1..box_height - 1 {
            print!("\x1b[{};{}H", start_row + i, start_col + 1);
            print!("{}", " ".repeat(box_width - 2));
        }

        // Draw the guide text centered in the box
        let line1 = "[toomuch] Resuming...";
        let line2 = "Press Ctrl-L to rerender your editor.";

        let line1_row = start_row + 1;
        let line2_row = start_row + 2;

        // Center line 1
        let line1_col = start_col + (box_width / 2).saturating_sub(line1.len() / 2);
        print!("\x1b[{};{}H", line1_row, line1_col);
        print!("\x1b[32m{}\x1b[0m", line1);

        // Center line 2
        let line2_col = start_col + (box_width / 2).saturating_sub(line2.len() / 2);
        print!("\x1b[{};{}H", line2_row, line2_col);
        print!("Press \x1b[1mCtrl-L\x1b[0m to rerender your editor.");

        io::stdout().flush().unwrap();
    } else {
        // Fallback if terminal size cannot be determined
        print!("\x1b[2J\x1b[H");
        print!("\x1b[32m[toomuch]\x1b[0m Resuming... Press \x1b[1mCtrl-L\x1b[0m to rerender your editor.\n\r");
        io::stdout().flush().unwrap();
    }
}

fn main() {
    let mut args: Vec<String> = env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("usage: toomuch <seconds> <command> [args...]");
        std::process::exit(1);
    }

    let timeout_secs: u64 = args.remove(0).parse().expect("invalid timeout");
    let timeout = Duration::from_secs(timeout_secs);
    let cmd_name = args.remove(0);
    
    let stdin_fd = io::stdin().as_raw_fd();
    let parent_pgrp = getpgrp();

    let mut child = Command::new(cmd_name)
        .args(args)
        .spawn()
        .expect("failed to spawn");

    let child_pid = Pid::from_raw(child.id() as i32);

    // 1. Create a new process group for the child
    let _ = setpgid(child_pid, child_pid);
    
    // 2. Give the child the terminal focus immediately
    let _ = tcsetpgrp(stdin_fd, child_pid);

    let start = Instant::now();
    let mut prompted = false;

    loop {
        // Check if child finished naturally
        match child.try_wait() {
            Ok(Some(status)) => {
                let _ = tcsetpgrp(stdin_fd, parent_pgrp);
                std::process::exit(status.code().unwrap_or(0));
            }
            Ok(None) => {}
            Err(e) => {
                eprintln!("Error waiting for child: {}", e);
                std::process::exit(1);
            }
        }

        if !prompted && start.elapsed() >= timeout {
            prompted = true;
            suspend_and_prompt(&mut child, child_pid, parent_pgrp, stdin_fd);
        }
    }
}

fn suspend_and_prompt(child: &mut Child, child_pid: Pid, parent_pgrp: Pid, stdin_fd: i32) {
    // A. Capture the terminal state (likely "Raw" mode if Vim/Emacs is running)
    let saved_editor_termios = Termios::from_fd(stdin_fd).ok();

    // B. Ignore background signals so the OS doesn't freeze the parent
    unsafe {
        libc::signal(libc::SIGTTOU, libc::SIG_IGN);
        libc::signal(libc::SIGTTIN, libc::SIG_IGN);
    }

    // C. Suspend the child process
    let _ = kill(child_pid, Signal::SIGSTOP);

    // D. Take back terminal ownership
    let _ = tcsetpgrp(stdin_fd, parent_pgrp);

    // E. Reset terminal to "Cooked" mode so the parent can read input normally
    if let Ok(mut parent_term) = Termios::from_fd(stdin_fd) {
        // ICANON: Buffer input until newline
        // ECHO: Show what the user types
        parent_term.c_lflag |= ICANON | ECHO; 

        // ICRNL: Map Carriage Return (^M) to Newline (\n)
        // This fixes the "^M" issue when hitting Enter
        parent_term.c_iflag |= termios::ICRNL;

        // Apply the settings immediately
        let _ = tcsetattr(stdin_fd, TCSADRAIN, &parent_term);
    }
    
    // F. The Prompt
    draw_centered_prompt();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    match input.trim().to_lowercase().as_str() {
        "c" => {
            let _ = child.kill();
            // CRITICAL: Clean up the mess before leaving
            reset_terminal(stdin_fd, parent_pgrp);
            eprintln!("\r\n[toomuch] Process killed.");
            std::process::exit(124);
        }
        _ => {
            // Update the prompt box to show guidance message
            update_prompt_with_guide();

            // 2. Restore the Editor's "Raw" mode
            if let Some(t) = saved_editor_termios {
                let _ = tcsetattr(stdin_fd, TCSADRAIN, &t);
            }

            // 3. Hand back to child
            let _ = tcsetpgrp(stdin_fd, child_pid);
            let _ = kill(child_pid, Signal::SIGCONT);
        }
    }

    // I. Restore signal defaults
    unsafe {
        libc::signal(libc::SIGTTOU, libc::SIG_DFL);
        libc::signal(libc::SIGTTIN, libc::SIG_DFL);
    }
}

fn reset_terminal(stdin_fd: i32, parent_pgrp: Pid) {
    unsafe {
        libc::signal(libc::SIGTTOU, libc::SIG_IGN);
    }
    // 1. Give terminal back to the shell's process group
    let _ = tcsetpgrp(stdin_fd, parent_pgrp);

    // 2. Force reset flags: Cooked mode, Echo on, Newline translation on
    if let Ok(mut term) = Termios::from_fd(stdin_fd) {
        term.c_lflag |= termios::ICANON | termios::ECHO;
        term.c_iflag |= termios::ICRNL;
        term.c_oflag |= termios::ONLCR; // Ensure \n is treated as \r\n on output
        let _ = tcsetattr(stdin_fd, termios::TCSANOW, &term);
    }
}
