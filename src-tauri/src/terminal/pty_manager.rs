// Full PTY terminal with async streaming
use std::sync::Arc;
use tokio::sync::Mutex;
use kael_terminal::{PtySession, TerminalManager as PtyBackend};

pub struct PtyTerminal {
    session: Arc<Mutex<Option<PtySession>>>,
}

impl Clone for PtyTerminal {
    fn clone(&self) -> Self {
        Self {
            session: Arc::clone(&self.session),
        }
    }
}

impl PtyTerminal {
    pub fn new() -> Self {
        Self {
            session: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn ensure_session(&self) -> Result<(), String> {
        let mut lock = self.session.lock().await;
        if lock.is_none() {
            let pty = PtyBackend::spawn_shell(None).map_err(|e| e.to_string())?;
            *lock = Some(pty);
        }
        Ok(())
    }

    pub async fn write_line(&self, line: &str) -> Result<(), String> {
        self.ensure_session().await?;
        let mut lock = self.session.lock().await;
        if let Some(ref mut pty) = *lock {
            let input = format!("{}\n", line);
            PtyBackend::write_input(pty, input.as_bytes()).await.map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub async fn get_output_receiver(&self) -> Result<async_channel::Receiver<Vec<u8>>, String> {
        self.ensure_session().await?;
        let lock = self.session.lock().await;
        if let Some(ref pty) = *lock {
            Ok(PtyBackend::output_stream(pty))
        } else {
            Err("No PTY session available".to_string())
        }
    }

    pub async fn kill(&self) -> Result<(), String> {
        let mut lock = self.session.lock().await;
        if let Some(ref mut pty) = *lock {
            pty.kill().map_err(|e| e.to_string())?;
            *lock = None;
        }
        Ok(())
    }
}
