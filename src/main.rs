#![deny(clippy::unwrap_used)]
#![deny(clippy::unwrap_used)]
#![warn(clippy::todo)]
use std::{env, fs, io::Write, vec};

use zula_core::{ShellState, ZulaError};

mod util;
use util::*;

const VER: &str = env!("CARGO_PKG_VERSION");

fn runtime(shell_state: &mut ShellState) -> Result<(), ZulaError> {
    write!(
        shell_state.stdout,
        "{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    )?;
    write!(shell_state.stdout, "\x1b[38;5;93mwelcome to zula.\x1b[38;5;5m\r\ntype \"zula\" for more information\x1b[0m\r\n\r\n")?;

    shell_state.stdout.flush()?;

    'l: loop {
        write!(shell_state.stdout, "{}", shell_state.get_header())?;
        shell_state.stdout.flush()?;
        let cmd = get_input(shell_state)?;
        match cmd.as_str() {
            "exit" => break 'l,
            "zula" => {
                write!(
                    shell_state.stdout,
                    "\r\n\x1b[38;5;93mzula version\x1b[38;5;5m {VER}\x1b[0m\r\n"
                )?;
                write!(
                    shell_state.stdout,
                    "\r\nother commands:\r\n\x1b[38;5;5mzula cfg:\x1b[0m show config information\r\n\x1b[38;5;5mzula reload:\x1b[0m reload .zularc\r\n\x1b[0m"
                )?;
            }
            "zula reload" => {
                gen_config(shell_state);
            }
            "zula cfg" => {
                let cfg = dirs::config_dir()
                    .map(|mut p| {
                        p.push("zula/.zularc");
                        p
                    })
                    .map(fs::read_to_string);
                shell_state.stdout.suspend_raw_mode()?;
                let cfg_loc = format!(
                    "{}/zula/.zularc\n\n",
                    dirs::config_dir()
                        .ok_or(ZulaError::InvalidDir)?
                        .to_string_lossy()
                );

                if let Some(Ok(info)) = cfg {
                    write!(shell_state.stdout, "{}", cfg_loc)?;
                    write!(shell_state.stdout, "{}", info)?;
                } else {
                    write!(
                        shell_state.stdout,
                        "config file not found\nlooking for {}\n",
                        cfg_loc
                    )?;
                }

                write!(shell_state.stdout, "\nplugins loaded:\n\n")?; 
                for plug in shell_state.plugin_names() {
                    println!("{}\n", plug);
                }

                shell_state.stdout.activate_raw_mode()?;
            }
            _ => {
                //command execution

                shell_state.stdout.suspend_raw_mode()?;
                match exec(
                    &cmd.trim(),
                    shell_state,
                    vec::Vec::with_capacity(shell_state.config.aliases.len()),
                ) {
                    Ok(()) => {}
                    Err(ZulaError::InvalidCmd(c)) => {
                        write!(shell_state.stdout, "unknown command: {c}")?
                    }
                    Err(ZulaError::Io(e)) => write!(shell_state.stdout, "program error: {e}")?,
                    Err(ZulaError::InvalidDir) => {
                        write!(shell_state.stdout, "directory does not exist")?
                    }
                    Err(ZulaError::RecursiveAlias) => write!(
                        shell_state.stdout,
                        "alias is infinitely recursive, so it cannot be expanded"
                    )?,
                    Err(ZulaError::InvalidPlugin) => write!(shell_state.stdout, "invalid plugin")?,
                    Err(ZulaError::Opaque(e)) => {write!(shell_state.stdout, "an external error occured: {e}")?}
                    _ => {}
                }
                shell_state.stdout.activate_raw_mode()?;
            }
        }

        shell_state.history.push(cmd.clone());
        write!(shell_state.stdout, "\r\n")?;
        shell_state.stdout.flush()?;
    }
    //println!("{}", cmd);

    Ok(())
}

fn init() -> Result<ShellState, ZulaError> {
    let mut shell_state = ShellState::new()?;

    gen_config(&mut shell_state);

    if let Ok(glob) = glob::glob(&format!("{}/*.so", cfg_dir("plugins").ok_or(ZulaError::InvalidDir)?)) {
        for entry in glob {
            if let Ok(path) = entry {
                match shell_state.load_plugin(path) {
                    Ok(()) => {},
                    Err(e) => write!(shell_state.stdout, "warning: plugin could not be loaded: {e}")?,
                };
            } 
        }
    } 

    Ok(shell_state)
}

fn gen_config(shell_state: &mut ShellState) {
    let cfg = cfg_dir(".zularc");

    if let Some(Ok(raw)) = cfg.map(fs::read_to_string) {
        for setting in raw.lines().filter(|l| l.starts_with('#')) {
            let args: Vec<_> = setting.split_whitespace().collect();
            match args[0] {
                "#alias" if args.len() >= 3 => {
                    shell_state
                        .config
                        .aliases
                        .insert(args[1].to_owned(), args[2..].join(" "));
                }
                "#bind" if args.len() >= 3 => {
                    //SAFETY: checking len ensures that the second element of the list will have at
                    //least one character
                    unsafe {
                        shell_state.config.hotkeys.insert(
                            args[1].chars().next().unwrap_unchecked(),
                            args[2..].join(" "),
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

fn cfg_dir(sub:&str) -> Option<String> {
    let cfg = if let Ok(mut s) = env::var("ZULA_CFG") {
        if s.ends_with('/') {
            s.push_str(sub);
        } else {
            s.push_str("/");
            s.push_str(sub);
        }
        Some(s)
    } else {
        dirs::config_dir().map(|s| {
            let mut s = s.to_string_lossy().to_string();
            s.push_str("/zula/");
            s.push_str(sub);
            s
        })
    };
    cfg
}

//execption handling
fn main() {
    let mut shell_state = match init() {
        Ok(v) => v,
        Err(e) => {
            println!(
                "zula has encountered an error and cannot be initialized properly.\nerror {e:?}"
            );
            return;
        }
    };

    match runtime(&mut shell_state) {
        Ok(()) => {}
        Err(e) => println!(
            "zula has encountered a fatal error and must exit.\nerror: {:?}",
            e
        ),
    }

    //explicitly exit raw mode
    //yes, this isnt needed, but it makes me feel better
    write!(
        shell_state.stdout,
        "{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    )
    .unwrap();
    drop(shell_state);
}
