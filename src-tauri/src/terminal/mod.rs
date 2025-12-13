// Terminal shim for the UI: sync API backed by PTY crate
mod pty_manager;
pub use pty_manager::PtyTerminal;

use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Clone)]
pub struct TerminalManager;

impl TerminalManager {
    pub fn new() -> Self { Self }

    pub fn run_command(&self, cmd: &str) -> String {
        // Use /bin/sh -c on Unix to allow pipelines and quoting.
        let output = Command::new("/bin/sh")
            .arg("-c")
            .arg(cmd)
            .output();

        match output {
            Ok(out) => {
                let mut s = String::new();
                s.push_str(&String::from_utf8_lossy(&out.stdout));
                if !out.stderr.is_empty() {
                    s.push_str(&String::from_utf8_lossy(&out.stderr));
                }
                let s = s.trim().to_string();
                if s.is_empty() { "(no output)".to_string() } else { s }
            }
            Err(e) => format!("Command error: {e}"),
        }
    }

    // Run sudo commands by feeding password via stdin (`sudo -S`).
    // Note: This is a synchronous helper intended for simple invocations like `sudo pacman -Syu`.
    // For complex pipelines under sudo, prefer a dedicated PTY stream.
    pub fn run_sudo_command(&self, cmdline: &str, password: &str) -> String {
        let trimmed = cmdline.trim();
        if !trimmed.starts_with("sudo ") && trimmed != "sudo" {
            return self.run_command(trimmed);
        }
        // naive split: "sudo <prog> <args...>"
        let mut parts = trimmed.split_whitespace();
        let _sudo = parts.next(); // "sudo"
        let prog = match parts.next() { Some(p) => p, None => return "sudo: no command".to_string() };
        let args: Vec<String> = parts.map(|s| s.to_string()).collect();

        let mut child = match Command::new("sudo")
            .arg("-S")
            .arg(prog)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => return format!("sudo spawn error: {e}"),
        };

        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(format!("{}\n", password).as_bytes());
        }

        match child.wait_with_output() {
            Ok(out) => {
                let mut s = String::new();
                s.push_str(&String::from_utf8_lossy(&out.stdout));
                if !out.stderr.is_empty() {
                    s.push_str(&String::from_utf8_lossy(&out.stderr));
                }
                let s = s.trim().to_string();
                if s.is_empty() { "(no output)".to_string() } else { s }
            }
            Err(e) => format!("sudo wait error: {e}"),
        }
    }
}

