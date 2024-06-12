//! Implements the dfx extension CLI commands for managing external canisters.

use crate::args::canister::Args;

/// The main entry point for the `dfx orbit` CLI.
pub fn main(args: Args) {
    match args {
        Args::Claim(_claim_args) => {
            todo!("Implement claiming a canister.");
        }
    }
}