//! Identity-provider clients for browser sign-in.
//!
//! We implement the IdP conversation ourselves rather than adopting a vendor's
//! optional UI layer (Cognito's hosted UI / managed login, Amplify UI,
//! `amazon-cognito-identity-js`). The platform is a core dependency; its UI
//! components are not — and owning this layer is what gives tight control over
//! the sign-in **sequence**, the **UX**, and the **logging/debugging** of every
//! failure.
//!
//! Pairs with [`crate::components::LoginScreen`] (the UI) and
//! [`crate::utils::webauthn`] (the passkey ceremonies).

mod cognito;

pub use cognito::*;
