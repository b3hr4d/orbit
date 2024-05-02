//! Core utility features for the control plane.

mod constants;
pub use constants::*;

mod memory;
pub use memory::*;

mod call_context;
pub use call_context::*;

mod config;
pub use config::*;

pub mod metrics;

pub mod middlewares;

#[cfg(not(test))]
pub use orbit_essentials::cdk as ic_cdk;
#[cfg(test)]
pub use orbit_essentials::cdk::mocks as ic_cdk;

#[cfg(not(test))]
pub use orbit_essentials::utils::generate_uuid_v4;
#[cfg(test)]
pub async fn generate_uuid_v4() -> uuid::Uuid {
    uuid::Uuid::new_v4()
}