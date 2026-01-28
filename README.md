# toomuch

`toomuch` is a GNU `timeout`-compatible command wrapper with **interactive suspend / resume** support for terminal applications like Vim, Emacs, and TUI programs.

Unlike traditional `timeout`, `toomuch` lets you **pause**, **inspect**, and **resume** long-running interactive commands safely.

---

## Features

- GNU `timeout`-compatible semantics
- Suspend / resume instead of blind killing
- Safe for Vim, Emacs, ncurses, TUIs
- SIGWINCH-aware redraw
- PTY-based tests (real terminal behavior)
- lean job control (tcsetpgrp, SIGSTOP/SIGCONT)

---

## Installation

```bash
cargo install toomuch
