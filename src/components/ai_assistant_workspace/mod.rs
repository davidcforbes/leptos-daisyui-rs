//! Provider-independent assistant projections and guarded, host-owned intents.
//!
//! Authorization, evidence assembly, transport, persistence and provider execution
//! belong to the consuming application. Presentation data is not an authorization
//! grant; unknown or incomplete host contracts fail closed.

mod artifact;
mod model;
mod settings;
mod types;

pub use artifact::*;
pub use model::*;
pub use settings::*;
pub use types::*;

#[cfg(test)]
mod tests;
