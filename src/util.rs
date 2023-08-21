use std::{
    io::{self, ErrorKind, Write},
    process::Command,
};
use termion::event::Key;
use termion::input::TermRead;

use zula_core::{ShellState, ZulaError};

pub(crate) fn get_input(state: &mut ShellState) -> Result<String, ZulaError> {
    let mut buf = String::new();
    let mut idx: usize = state.history.len();

    for c in io::stdin().keys() {
        match c? {
            Key::Alt(c) => {
                if let Some(cmd) = state.config.hotkeys.get(&c) {
                    return Ok(cmd.clone());
                }
            }
            Key::Char('\n') => break,
            Key::Char('\x09') => {} //tab
            Key::Char(c) => {
                buf.push(c);
                print!("{}", c);
            }
            //TODO: moving cursor
            //Key::Left => print!("\x08"),
            //Key::Right => print!("→"),
            Key::Up if !state.history.is_empty() && idx > 0 => {
                idx -= 1;
                buf = state.history[idx].clone();
                //this is so fucking stupid but im not gonna change it right now
                print!(
                    "{}{}{}{}",
                    termion::clear::CurrentLine,
                    termion::cursor::Left(u16::MAX),
                    state.get_header(),
                    buf
                );
            }
            Key::Down if idx + 1 < state.history.len() => {
                idx += 1;
                buf = state.history[idx].clone();
                //this is so fucking stupid but im not gonna change it right now
                print!(
                    "{}{}{}{}",
                    termion::clear::CurrentLine,
                    termion::cursor::Left(u16::MAX),
                    state.get_header(),
                    buf
                );
            }
            Key::Backspace => {
                if buf.pop().is_some() {
                    print!("\x08 \x08")
                }
            }
            _ => {}
        }
        state.stdout.flush()?;
    }
    print!("\r\n");

    Ok(buf)
}

#[inline]
pub(crate) fn home() -> Result<String, ZulaError> {
    dirs::home_dir()
        .ok_or(ZulaError::CommandEmpty)
        .map(|s| s.to_string_lossy().to_string())
}

pub(crate) fn exec(
    raw: &str,
    state: &mut ShellState,
    mut walked: Vec<String>,
) -> Result<(), ZulaError> {
    let mut args: Vec<String> = vec![];
    let mut quoted = false;
    let mut string = String::with_capacity(raw.len());

    for c in raw.chars() {
        if c != ' ' {
            if c == '"' {
                quoted = !quoted;
            } else {
                string.push(c)
            }
        } else if quoted == false {
            args.push(string.clone());
            string.clear();
        } else {
            string.push(c)
        }
    }
    args.push(string);

    if args.is_empty() {
        return Err(ZulaError::CommandEmpty);
    }

    //aliases
    if args[0].starts_with('!') {
        args[0].remove(0);
    } else {
        if let Some(c) = state.config.aliases.get(&args[0]) {
            if !walked.contains(c) {
                args.remove(0);
                let cmd_raw = format!("{c} {}", args.join(" "));
                walked.push(c.clone());
                return exec(&cmd_raw, state, walked);
            } else {
                return Err(ZulaError::RecursiveAlias);
            }
        }
    }

    let home = home()?;
    //TODO: this could be better maybe
    args.iter_mut().for_each(|s| {
        if s.starts_with('~') {
            *s = s.replacen('~', &home, 1)
        }
    });
    args.iter_mut().for_each(|s| {
        if s.starts_with('\\') {
            s.remove(0);
        }
    });

    state.exec(&args[0], &args[1..])?;

    Ok(())
}
