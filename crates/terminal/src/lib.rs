use std::io::Read;

use anyhow::Result;
use async_channel::{Receiver, Sender};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use thiserror::Error;
use tracing::{error};

#[derive(Error, Debug)]
pub enum TerminalError {
    #[error("PTY spawn error: {0}")]
    Spawn(String),
}

#[derive(Clone)]
pub struct PtySessionHandle {
    #[allow(dead_code)]
    tx: Sender<Vec<u8>>, // stdout/stderr chunks to UI
    rx: Receiver<Vec<u8>>, // for potential stdin echo or control
}

pub struct PtySession {
    pub child: Box<dyn portable_pty::Child + Send>,
    pub master: Box<dyn portable_pty::MasterPty + Send>,
    pub writer: Box<dyn std::io::Write + Send>,
    pub reader_task: tokio::task::JoinHandle<()>,
    pub handle: PtySessionHandle,
}

impl PtySession {
    pub fn kill(&mut self) -> Result<()> {
        self.child.kill().map_err(|e| anyhow::anyhow!(e))
    }
}

pub struct TerminalManager;

impl TerminalManager {
    pub fn spawn_shell(size: Option<PtySize>) -> Result<PtySession> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(size.unwrap_or(PtySize {
            rows: 24,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        }))
        .map_err(|e| anyhow::anyhow!(e))?;

        let cmd = CommandBuilder::new("/bin/sh");
        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| anyhow::anyhow!(e))?;

        let reader = pair.master.try_clone_reader().map_err(|e| anyhow::anyhow!(e))?;
        let writer = pair.master.take_writer().map_err(|e| anyhow::anyhow!(e))?;
        let master = pair.master;

        let (tx, rx) = async_channel::bounded::<Vec<u8>>(128);
        let tx_reader = tx.clone();

        let reader_task = tokio::task::spawn_blocking(move || {
            let mut buf = [0u8; 4096];
            let mut r = reader;
            loop {
                match r.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        let chunk = buf[..n].to_vec();
                        if let Err(e) = tx_reader.send_blocking(chunk) {
                            eprintln!("PTY reader send error: {e}");
                            break;
                        }
                    }
                    Ok(_) => break,
                    Err(e) => {
                        eprintln!("PTY read error: {e}");
                        break;
                    }
                }
            }
        });

        Ok(PtySession {
            child,
            master,
            writer,
            reader_task,
            handle: PtySessionHandle { tx, rx },
        })
    }

    pub async fn write_input(session: &mut PtySession, data: &[u8]) -> Result<()> {
        use std::io::Write;
        session.writer.write_all(data).map_err(|e| anyhow::anyhow!(e))?;
        session.writer.flush().map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn resize(session: &mut PtySession, size: PtySize) -> Result<()> {
        session.master.resize(size).map_err(|e| anyhow::anyhow!(format!("{e}")))
    }

    pub fn output_stream(session: &PtySession) -> Receiver<Vec<u8>> {
        session.handle.rx.clone()
    }
}
