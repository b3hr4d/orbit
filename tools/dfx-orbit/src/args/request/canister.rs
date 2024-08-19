//! CLI arguments for `dfx-orbit canister`.

use crate::DfxOrbit;
use anyhow::{bail, Context};
use clap::{Parser, Subcommand, ValueEnum};

use orbit_station_api::{
    CallExternalCanisterOperationInput, CanisterInstallMode, CanisterMethodDTO,
    ChangeExternalCanisterOperationInput, GetRequestResponse, RequestOperationDTO,
    RequestOperationInput,
};
use sha2::{Digest, Sha256};
use slog::info;

// TODO: Support Canister create + integration test
// TODO: Support Canister install check
// TODO: Canister get response functionality

/// Request canister operations through Orbit
#[derive(Debug, Clone, Parser)]
pub struct RequestCanisterArgs {
    /// The operation to request
    #[clap(subcommand)]
    action: RequestCanisterActionArgs,
}

#[derive(Debug, Clone, Subcommand)]
#[clap(version, about, long_about = None)]
pub enum RequestCanisterActionArgs {
    /// Request to upgrade the canister wasm
    Install(RequestCanisterInstallArgs),
    /// Request to call a canister method
    Call(RequestCanisterCallArgs),
}

impl RequestCanisterArgs {
    /// Converts the CLI arg type into the equivalent Orbit API type.
    pub(crate) fn into_create_request_input(
        self,
        dfx_orbit: &DfxOrbit,
    ) -> anyhow::Result<RequestOperationInput> {
        self.action.into_create_request_input(dfx_orbit)
    }
}

impl RequestCanisterActionArgs {
    /// Converts the CLI arg type into the equivalent Orbit API type.
    pub(crate) fn into_create_request_input(
        self,
        dfx_orbit: &DfxOrbit,
    ) -> anyhow::Result<RequestOperationInput> {
        match self {
            RequestCanisterActionArgs::Install(change_args) => {
                change_args.into_create_request_input(dfx_orbit)
            }
            RequestCanisterActionArgs::Call(call_args) => {
                call_args.into_create_request_input(dfx_orbit)
            }
        }
    }
}

/// Requests that a call be made to a canister.
#[derive(Debug, Clone, Parser)]
pub struct RequestCanisterCallArgs {
    /// The canister name or ID.
    canister: String,
    /// The name of the method to call.
    method_name: String,
    /// The argument to pass to the method.
    argument: Option<String>,
    // TODO: The format of the argument.
    // #[clap(short, long)]
    // r#type: Option<CandidFormat>,
    /// Pass the argument as a file.
    #[clap(short = 'f', long, conflicts_with = "argument")]
    arg_file: Option<String>,
    /// Specifies the amount of cycles to send on the call.
    #[clap(short, long)]
    with_cycles: Option<u64>,
}

impl RequestCanisterCallArgs {
    /// Converts the CLI arg stype into the equivalent Orbit API type.
    pub(crate) fn into_create_request_input(
        self,
        dfx_orbit: &DfxOrbit,
    ) -> anyhow::Result<RequestOperationInput> {
        let canister_id = dfx_orbit.canister_id(&self.canister)?;
        let arg = candid_from_string_or_file(&self.argument, &self.arg_file)?;

        Ok(RequestOperationInput::CallExternalCanister(
            CallExternalCanisterOperationInput {
                validation_method: None,
                execution_method: CanisterMethodDTO {
                    canister_id,
                    method_name: self.method_name,
                },
                arg,
                execution_method_cycles: self.with_cycles,
            },
        ))
    }

    pub(crate) fn verify(
        &self,
        dfx_orbit: &DfxOrbit,
        request: &GetRequestResponse,
    ) -> anyhow::Result<()> {
        let canister_id = dfx_orbit.canister_id(&self.canister)?;
        let arg = candid_from_string_or_file(&self.argument, &self.arg_file)?;
        let arg_checksum = arg.map(|arg| hex::encode(Sha256::digest(arg)));

        let RequestOperationDTO::CallExternalCanister(op) = &request.request.operation else {
            bail!("This request is not an external canister call");
        };
        if op.execution_method.canister_id != canister_id {
            bail!(
                "Canister id of request \"{}\" does not match expected id",
                op.execution_method.canister_id
            )
        }
        if op.execution_method.method_name != self.method_name {
            bail!(
                "The request targets another method: \"{}\"",
                op.execution_method.method_name
            )
        }
        if op.arg_checksum != arg_checksum {
            info!(
                dfx_orbit.logger,
                "Request argument: {}",
                display_arg_checksum(&op.arg_checksum)
            );
            info!(
                dfx_orbit.logger,
                "Local argument:   {}",
                display_arg_checksum(&arg_checksum)
            );
            bail!("Argument checksum does not match");
        }
        if op.execution_method_cycles != self.with_cycles {
            bail!("Attached cycles do not match");
        }

        Ok(())
    }
}

/// Requests that a canister be installed or updated.  Equivalent to `orbit_station_api::CanisterInstallMode`.
#[derive(Debug, Clone, Parser)]
pub struct RequestCanisterInstallArgs {
    /// The canister name or ID.
    canister: String,
    /// The installation mode.
    #[clap(long, value_enum, rename_all = "kebab-case", default_value = "install")]
    mode: CanisterInstallModeArgs,
    /// The path to the Wasm file to install.
    #[clap(short, long)]
    wasm: String,
    /// The argument to pass to the canister.
    #[clap(short, long, conflicts_with = "arg_file")]
    argument: Option<String>,
    /// The path to a file containing the argument to pass to the canister.
    #[clap(short = 'f', long, conflicts_with = "arg")]
    arg_file: Option<String>,
}

impl RequestCanisterInstallArgs {
    /// Converts the CLI arg type into the equivalent Orbit API type.
    pub(crate) fn into_create_request_input(
        self,
        dfx_orbit: &DfxOrbit,
    ) -> anyhow::Result<RequestOperationInput> {
        let canister_id = dfx_orbit.canister_id(&self.canister)?;

        let operation = {
            let module = std::fs::read(self.wasm)
                .expect("Could not read Wasm file")
                .to_vec();
            let arg = if let Some(file) = self.arg_file {
                Some(
                    std::fs::read(file)
                        .expect("Could not read argument file")
                        .to_vec(),
                )
            } else {
                self.argument.map(|arg| arg.as_bytes().to_vec())
            };
            let mode = self.mode.into();
            ChangeExternalCanisterOperationInput {
                canister_id,
                mode,
                module,
                arg,
            }
        };
        Ok(RequestOperationInput::ChangeExternalCanister(operation))
    }

    pub(crate) fn verify(
        &self,
        dfx_orbit: &DfxOrbit,
        request: &GetRequestResponse,
    ) -> anyhow::Result<()> {
        let canister_id = dfx_orbit.canister_id(&self.canister)?;
        let arg = candid_from_string_or_file(&self.argument, &self.arg_file)?;
        let arg_checksum = arg.map(|arg| hex::encode(Sha256::digest(arg)));

        let RequestOperationDTO::ChangeExternalCanister(op) = &request.request.operation else {
            bail!("This request is not a change external canister request");
        };
        if CanisterInstallModeArgs::from(op.mode.clone()) != self.mode {
            bail!("");
        }

        todo!()
    }
}

/// Canister installation mode equivalent to `dfx canister install --mode XXX` and `orbit_station_api::CanisterInstallMode`.
#[derive(Copy, Clone, Eq, PartialEq, Debug, ValueEnum)]
pub enum CanisterInstallModeArgs {
    /// Corresponds to `dfx canister install`
    Install,
    /// Corresponds to `dfx canister reinstall`
    Reinstall,
    /// Corresponds to `dfx canister upgrade`
    Upgrade,
}

impl From<CanisterInstallModeArgs> for CanisterInstallMode {
    fn from(mode: CanisterInstallModeArgs) -> Self {
        match mode {
            CanisterInstallModeArgs::Install => Self::Install,
            CanisterInstallModeArgs::Reinstall => Self::Reinstall,
            CanisterInstallModeArgs::Upgrade => Self::Upgrade,
        }
    }
}

impl From<CanisterInstallMode> for CanisterInstallModeArgs {
    fn from(mode: CanisterInstallMode) -> Self {
        match mode {
            CanisterInstallMode::Install => Self::Install,
            CanisterInstallMode::Reinstall => Self::Reinstall,
            CanisterInstallMode::Upgrade => Self::Upgrade,
        }
    }
}

fn candid_from_string_or_file(
    arg_string: &Option<String>,
    arg_path: &Option<String>,
) -> anyhow::Result<Option<Vec<u8>>> {
    // TODO: It would be really nice to be able to use `blob_from_arguments(..)` here, as in dfx, to geta ll the nice things such as help composing the argument.
    // First try to read the argument file, if it was provided
    Ok(arg_path
        .as_ref()
        .map(std::fs::read_to_string)
        .transpose()?
        // Otherwise use the argument from the command line
        .or_else(|| arg_string.clone())
        // Parse the candid
        .map(|arg_string| {
            candid_parser::parse_idl_args(&arg_string)
                .with_context(|| "Invalid Candid values".to_string())?
                .to_bytes()
        })
        .transpose()?)
}

fn display_arg_checksum(arg: &Option<String>) -> String {
    arg.as_ref()
        .map(|s| format!("0x{}", s))
        .unwrap_or(String::from("No argument"))
}
