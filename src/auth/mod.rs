//! Authentication module — JWT middleware and Argon2 password hashing.

pub mod argon;
pub mod jwt;

pub use jwt::{AdminClaims, Claims, JwtConfig, StoreClaims};
