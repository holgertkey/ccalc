use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

/// A file handle opened via `fopen`.
enum FileHandle {
    Read(BufReader<File>),
    Write(File),
}

/// File descriptor table for the REPL session.
///
/// Lives in the binary layer and is passed into `eval_with_io`.
/// File descriptors 1 (stdout) and 2 (stderr) are virtual — not stored here,
/// handled by `write_to_fd` directly. User-opened files start at fd 3.
pub struct IoContext {
    handles: HashMap<i32, FileHandle>,
    next_fd: i32,
}

impl IoContext {
    pub fn new() -> Self {
        Self {
            handles: HashMap::new(),
            next_fd: 3,
        }
    }

    /// Opens a file and returns a new file descriptor, or -1 on failure.
    /// Supported modes: `"r"`, `"w"`, `"a"`, `"r+"`.
    pub fn fopen(&mut self, path: &str, mode: &str) -> i32 {
        let handle = match mode {
            "r" => File::open(path).map(|f| FileHandle::Read(BufReader::new(f))),
            "w" => File::create(path).map(|f| FileHandle::Write(f)),
            "a" => OpenOptions::new()
                .append(true)
                .create(true)
                .open(path)
                .map(|f| FileHandle::Write(f)),
            "r+" => OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .map(|f| FileHandle::Write(f)),
            _ => return -1,
        };
        match handle {
            Ok(h) => {
                let fd = self.next_fd;
                self.handles.insert(fd, h);
                self.next_fd += 1;
                fd
            }
            Err(_) => -1,
        }
    }

    /// Closes a file descriptor. Returns 0 on success, -1 if fd is unknown.
    pub fn fclose(&mut self, fd: i32) -> i32 {
        if self.handles.remove(&fd).is_some() { 0 } else { -1 }
    }

    /// Closes all open file handles.
    pub fn fclose_all(&mut self) {
        self.handles.clear();
    }

    /// Reads one line from fd, stripping the trailing newline (`fgetl` semantics).
    /// Returns `None` at EOF or on error.
    pub fn fgetl(&mut self, fd: i32) -> Option<String> {
        match self.handles.get_mut(&fd)? {
            FileHandle::Read(reader) => {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => None,
                    Ok(_) => {
                        if line.ends_with('\n') {
                            line.pop();
                        }
                        if line.ends_with('\r') {
                            line.pop();
                        }
                        Some(line)
                    }
                    Err(_) => None,
                }
            }
            FileHandle::Write(_) => None,
        }
    }

    /// Reads one line from fd, keeping the trailing newline (`fgets` semantics).
    /// Returns `None` at EOF or on error.
    pub fn fgets(&mut self, fd: i32) -> Option<String> {
        match self.handles.get_mut(&fd)? {
            FileHandle::Read(reader) => {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => None,
                    Ok(_) => Some(line),
                    Err(_) => None,
                }
            }
            FileHandle::Write(_) => None,
        }
    }

    /// Writes a string to a file descriptor.
    /// fd 1 = stdout, fd 2 = stderr; all others must be in the handle table.
    pub fn write_to_fd(&mut self, fd: i32, s: &str) -> Result<(), String> {
        match fd {
            1 => {
                print!("{s}");
                std::io::stdout().flush().ok();
                Ok(())
            }
            2 => {
                eprint!("{s}");
                std::io::stderr().flush().ok();
                Ok(())
            }
            _ => match self.handles.get_mut(&fd) {
                Some(FileHandle::Write(f)) => f
                    .write_all(s.as_bytes())
                    .map_err(|e| format!("fprintf: write error: {e}")),
                Some(FileHandle::Read(_)) => {
                    Err(format!("fprintf: fd {fd} is not open for writing"))
                }
                None => Err(format!("fprintf: invalid file descriptor {fd}")),
            },
        }
    }
}
