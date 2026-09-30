//! Input sources for the REPL: a plain line reader, the Lix-compatible ENQ
//! protocol, and a rustyline editor.

use std::io::{BufRead, BufReader, IsTerminal, Write};

/// Reads submissions into a buffer.
pub(crate) trait Interacter {
    /// Read one chunk. `first` is true for the first chunk of a submission.
    /// Returns `Ok(false)` on end of input.
    fn read(&mut self, buf: &mut String, first: bool) -> std::io::Result<bool>;
    /// Called once when the REPL exits.
    fn finish(&mut self) {}
}

/// Reads lines without a line editor; prints a prompt only on a TTY.
pub(crate) struct PlainInteracter {
    stdin: BufReader<std::io::Stdin>,
    interactive: bool,
}

impl PlainInteracter {
    pub(crate) fn new() -> Self {
        PlainInteracter {
            stdin: BufReader::new(std::io::stdin()),
            interactive: std::io::stdin().is_terminal(),
        }
    }
}

impl Interacter for PlainInteracter {
    fn read(&mut self, buf: &mut String, first: bool) -> std::io::Result<bool> {
        if self.interactive {
            let mut out = std::io::stdout();
            out.write_all(if first { b"tsnix> " } else { b"    ..> " })?;
            out.flush()?;
        }
        let mut line = String::new();
        if self.stdin.read_line(&mut line)? == 0 {
            return Ok(false);
        }
        buf.push_str(&line);
        Ok(true)
    }
}

/// Lix `repl-automation`: emit ENQ (U+0005) before every read.
pub(crate) struct EnqInteracter {
    stdin: BufReader<std::io::Stdin>,
}

impl EnqInteracter {
    pub(crate) fn new() -> Self {
        EnqInteracter {
            stdin: BufReader::new(std::io::stdin()),
        }
    }
}

impl Interacter for EnqInteracter {
    fn read(&mut self, buf: &mut String, _first: bool) -> std::io::Result<bool> {
        let mut out = std::io::stdout();
        out.write_all(b"\x05")?;
        out.flush()?;
        let mut line = String::new();
        if self.stdin.read_line(&mut line)? == 0 {
            return Ok(false);
        }
        buf.push_str(&line);
        Ok(true)
    }
}

/// Interactive line editor backed by `rustyline`.
pub(crate) struct ReadlineInteracter {
    editor: rustyline::DefaultEditor,
    history: Option<std::path::PathBuf>,
}

impl ReadlineInteracter {
    pub(crate) fn new() -> Self {
        let editor = rustyline::DefaultEditor::new().expect("failed to initialise the line editor");
        let history = history_path();
        let mut interacter = ReadlineInteracter { editor, history };
        if let Some(path) = &interacter.history {
            let _ = interacter.editor.load_history(path);
        }
        interacter
    }
}

impl Interacter for ReadlineInteracter {
    fn read(&mut self, buf: &mut String, first: bool) -> std::io::Result<bool> {
        use rustyline::error::ReadlineError;

        let prompt = if first { "tsnix> " } else { "    ..> " };
        match self.editor.readline(prompt) {
            Ok(line) => {
                if !line.trim().is_empty() {
                    let _ = self.editor.add_history_entry(line.as_str());
                }
                buf.push_str(&line);
                buf.push('\n');
                Ok(true)
            }
            Err(ReadlineError::Interrupted) => {
                buf.clear();
                Ok(true)
            }
            Err(ReadlineError::Eof) => Ok(false),
            Err(error) => Err(std::io::Error::other(error)),
        }
    }

    fn finish(&mut self) {
        if let Some(path) = &self.history {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = self.editor.save_history(path);
        }
    }
}

fn history_path() -> Option<std::path::PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| std::path::PathBuf::from(home).join(".local/share"))
        })?;
    Some(base.join("tsnix").join("history"))
}
