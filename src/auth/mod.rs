#![allow(unused_imports)]
//! Authentication module — JWT middleware, Argon2 password hashing,
//! and OAuth2 social login providers.

pub mod argon;
pub mod jwt;

// OAuth2 social login
pub mod oauth;
pub mod oauth_service;
pub mod providers;

pub use jwt::{AdminClaims, Claims, JwtConfig, StoreClaims};
pub use oauth::{OAuthProfile, OAuthProvider, OAuthTokens};
pub use oauth_service::OAuthService;
