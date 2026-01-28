use std::sync::atomic::{AtomicBool, Ordering};

static WINCH: AtomicBool = AtomicBool::new(false);

pub fn install_sigwinch() {
    unsafe {
        libc::signal(libc::SIGWINCH, handle_sigwinch as usize);
    }
}

extern "C" fn handle_sigwinch(_: i32) {
    WINCH.store(true, Ordering::SeqCst);
}

pub fn winch_triggered() -> bool {
    WINCH.swap(false, Ordering::SeqCst)
}
