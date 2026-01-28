use crate::{signal::winch_triggered, ui::draw_centered_prompt};
use std::io;

pub enum PromptAction {
    Close,
    Resume,
}

pub fn prompt_user() -> PromptAction {
    draw_centered_prompt();
    let mut input = String::new();

    loop {
        if winch_triggered() {
            draw_centered_prompt();
        }

        if io::stdin().read_line(&mut input).is_ok() {
            break;
        }
    }

    match input.trim().to_lowercase().as_str() {
        "c" => PromptAction::Close,
        _ => PromptAction::Resume,
    }
}
