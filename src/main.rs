use cursive::views::{Dialog, TextView};
use cursive::{Cursive, CursiveExt};
use cursive::theme::{Theme};
use std::process::Command;

fn main() {
    let mut siv: Cursive = Cursive::default();
    siv.set_theme(Theme::terminal_default());

    let output = Command::new("ls").args(&["-lah"]).output().expect("Failed to execute command!");
    let output_str = String::from_utf8_lossy(&output.stdout);

    siv.add_layer(
        Dialog::around(TextView::new(output_str))
        .title("Command Output")
        .button("Quit", |s|s.quit())
    );
    siv.run();    
}

