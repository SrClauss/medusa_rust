//! Concrete OAuth2 provider implementations.
//!
//! Each provider is compiled only when its Cargo feature is enabled:
//! - `auth-google`   → `GoogleOAuthProvider`
//! - `auth-facebook` → `FacebookOAuthProvider`
//! - `auth-github`   → `GitHubOAuthProvider`

pub mod google;
pub mod facebook;
pub mod github;
