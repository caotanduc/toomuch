use std::io::{self, Write};
use terminal_size::{terminal_size, Height, Width};

pub fn draw_centered_prompt() {
    if let Some((Width(w), Height(h))) = terminal_size() {
        let bw = 60;
        let bh = 5;

        let sr = (h as usize / 2).saturating_sub(bh / 2);
        let sc = (w as usize / 2).saturating_sub(bw / 2);

        print!("\x1b[2J");

        print!("\x1b[{};{}H┌{}┐", sr, sc, "─".repeat(bw - 2));
        for i in 1..bh - 1 {
            print!("\x1b[{};{}H│{}│", sr + i, sc, " ".repeat(bw - 2));
        }
        print!("\x1b[{};{}H└{}┘", sr + bh - 1, sc, "─".repeat(bw - 2));

        let title = "[toomuch] Time limit exceeded.";
        let opts = "(c) close | (r) resume";

        print!(
            "\x1b[{};{}H\x1b[31;1m{}\x1b[0m",
            sr + 1,
            sc + bw / 2 - title.len() / 2,
            title
        );

        print!("\x1b[{};{}H{}", sr + 2, sc + bw / 2 - opts.len() / 2, opts);

        print!("\x1b[{};{}H> ", sr + 3, sc + bw / 2 - 1);

        io::stdout().flush().unwrap();
    }
}

pub fn update_prompt_with_guide() {
    if let Some((Width(w), Height(h))) = terminal_size() {
        let bw = 60;
        let bh = 5;

        let sr = (h as usize / 2).saturating_sub(bh / 2);
        let sc = (w as usize / 2).saturating_sub(bw / 2);

        for i in 1..bh - 1 {
            print!("\x1b[{};{}H{}", sr + i, sc + 1, " ".repeat(bw - 2));
        }

        let l1 = "[toomuch] Resuming...";
        let l2 = "Press Ctrl-L to rerender your editor.";

        print!(
            "\x1b[{};{}H\x1b[32m{}\x1b[0m",
            sr + 1,
            sc + bw / 2 - l1.len() / 2,
            l1
        );

        print!("\x1b[{};{}H{}", sr + 2, sc + bw / 2 - l2.len() / 2, l2);

        io::stdout().flush().unwrap();
    }
}
