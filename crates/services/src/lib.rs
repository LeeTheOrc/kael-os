/// Integration traits and adapters for Firebase, GitHub, Google, and VS Code.
use anyhow::Result;
use async_trait::async_trait;

pub mod llm;
pub mod system_context;
pub mod firebase;

/// Trait for auth providers (Firebase, GitHub, Google).
#[async_trait]
pub trait AuthProvider: Send + Sync {
    async fn authenticate(&self) -> Result<String>;
    async fn refresh_token(&self, token: &str) -> Result<String>;
}

/// Trait for AI backend (Firebase ML, OpenAI, local LLM).
#[async_trait]
pub trait AIBackend: Send + Sync {
    async fn complete(&self, prompt: &str) -> Result<String>;
}

/// Trait for version control (GitHub).
#[async_trait]
pub trait VCSBackend: Send + Sync {
    async fn list_repos(&self) -> Result<Vec<String>>;
    async fn push_changes(&self, repo: &str, message: &str) -> Result<()>;
}

/// Trait for editor integration (VS Code).
#[async_trait]
pub trait EditorBackend: Send + Sync {
    async fn open_in_editor(&self, file_path: &str) -> Result<()>;
    async fn get_current_file(&self) -> Result<Option<String>>;
}

/// Mock offline implementations for each provider.
pub mod mocks {
    use super::*;

    pub struct MockAuthProvider;

    #[async_trait]
    impl AuthProvider for MockAuthProvider {
        async fn authenticate(&self) -> Result<String> {
            Ok("mock_token".to_string())
        }

        async fn refresh_token(&self, _token: &str) -> Result<String> {
            Ok("mock_refreshed_token".to_string())
        }
    }

    pub struct MockAIBackend;

    #[async_trait]
    impl AIBackend for MockAIBackend {
        async fn complete(&self, prompt: &str) -> Result<String> {
            Ok(format!("Mock response to: {}", prompt))
        }
    }

    pub struct MockVCSBackend;

    #[async_trait]
    impl VCSBackend for MockVCSBackend {
        async fn list_repos(&self) -> Result<Vec<String>> {
            Ok(vec!["kael-os".to_string()])
        }

        async fn push_changes(&self, _repo: &str, _message: &str) -> Result<()> {
            Ok(())
        }
    }

    pub struct MockEditorBackend;

    #[async_trait]
    impl EditorBackend for MockEditorBackend {
        async fn open_in_editor(&self, _file_path: &str) -> Result<()> {
            Ok(())
        }

        async fn get_current_file(&self) -> Result<Option<String>> {
            Ok(None)
        }
    }
}

/// Service registry combining all backends.
pub struct ServiceRegistry {
    pub auth: Box<dyn AuthProvider>,
    pub ai: Box<dyn AIBackend>,
    pub vcs: Box<dyn VCSBackend>,
    pub editor: Box<dyn EditorBackend>,
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self {
            auth: Box::new(mocks::MockAuthProvider),
            ai: Box::new(mocks::MockAIBackend),
            vcs: Box::new(mocks::MockVCSBackend),
            editor: Box::new(mocks::MockEditorBackend),
        }
    }
}
