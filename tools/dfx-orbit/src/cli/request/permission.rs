//! Implements the dfx extension CLI commands for making requests.
pub mod canister;

use crate::args::request::permission::Args;

/// The main entry point for the `dfx orbit` CLI.
pub async fn main(
    args: Args,
) -> anyhow::Result<Result<orbit_station_api::CreateRequestResponse, orbit_station_api::ApiErrorDTO>>
{
    match args {
        Args::Canister(canister_args) => canister::main(canister_args).await,
    }
}
