// PTY implementation scaffold (future streaming)
// This is a placeholder for a proper PTY; we will wire this
// into ChatPanel to stream output chunks in real-time.

pub struct PTY;

impl PTY {
	pub fn new() -> Self { Self }

	// TODO: spawn an interactive shell in a PTY and provide a callback per chunk
	// pub fn spawn_stream<F: FnMut(String) + Send + 'static>(&self, cmd: &str, mut on_chunk: F) { ... }
}
