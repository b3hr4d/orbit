use crate::{
    core::middlewares::{authorize, call_context},
    mappers::HelperMapper,
    models::access_control::{ResourceSpecifier, ResourceType, UserActionSpecifier},
    services::UserService,
};
use ic_canister_core::api::ApiResult;
use ic_canister_macros::with_middleware;
use ic_cdk_macros::{query, update};
use lazy_static::lazy_static;
use wallet_api::{
    ConfirmUserIdentityInput, ConfirmUserIdentityResponse, GetUserInput, GetUserResponse,
    ListUsersInput, ListUsersResponse, MeResponse,
};

// Canister entrypoints for the controller.
#[update(name = "confirm_user_identity")]
async fn confirm_user_identity(
    input: ConfirmUserIdentityInput,
) -> ApiResult<ConfirmUserIdentityResponse> {
    CONTROLLER.confirm_user_identity(input).await
}

#[query(name = "get_user")]
async fn get_user(input: GetUserInput) -> ApiResult<GetUserResponse> {
    CONTROLLER.get_user(input).await
}

#[query(name = "list_users")]
async fn list_users(input: ListUsersInput) -> ApiResult<ListUsersResponse> {
    CONTROLLER.list_users(input).await
}

#[query(name = "me")]
async fn me() -> ApiResult<MeResponse> {
    CONTROLLER.me().await
}

// Controller initialization and implementation.
lazy_static! {
    static ref CONTROLLER: UserController = UserController::new(UserService::default());
}

#[derive(Debug)]
pub struct UserController {
    user_service: UserService,
}

impl UserController {
    fn new(user_service: UserService) -> Self {
        Self { user_service }
    }

    /// Confirms the user identity if the provided user has the caller identity as unconfirmed.
    ///
    /// No authorization required since the user will be calling this
    /// with a new identity that is not yet confirmed.
    async fn confirm_user_identity(
        &self,
        input: ConfirmUserIdentityInput,
    ) -> ApiResult<ConfirmUserIdentityResponse> {
        let ctx = call_context();
        let user = self
            .user_service
            .confirm_user_identity(input, ctx.caller())
            .await?
            .into();

        Ok(ConfirmUserIdentityResponse { user })
    }

    #[with_middleware(
        guard = "authorize",
        context = "call_context",
        args = [ResourceSpecifier::from(&input)],
        is_async = true
    )]
    async fn get_user(&self, input: GetUserInput) -> ApiResult<GetUserResponse> {
        let user = self
            .user_service
            .get_user(HelperMapper::to_uuid(input.user_id)?.as_bytes())?
            .into();

        Ok(GetUserResponse { user })
    }

    #[with_middleware(
        guard = "authorize",
        context = "call_context",
        args = [ResourceSpecifier::Common(ResourceType::User, UserActionSpecifier::List)],
        is_async = true
    )]
    async fn list_users(&self, input: ListUsersInput) -> ApiResult<ListUsersResponse> {
        let list = self.user_service.list_users(input)?;

        Ok(ListUsersResponse {
            users: list.items.into_iter().map(Into::into).collect(),
            next_offset: list.next_offset,
        })
    }

    /// Returns the user that is calling this endpoint.
    ///
    /// No authorization required since this endpoint only exposes the user associated with the caller identity.
    /// If the caller does not have a user associated with the identity, an error will be returned.
    async fn me(&self) -> ApiResult<MeResponse> {
        let ctx = call_context();
        let user = self.user_service.get_user_by_identity(&ctx.caller())?;

        let privileges = self
            .user_service
            .get_user_privileges_by_identity(&ctx.caller())
            .await?;

        Ok(MeResponse {
            me: user.into(),
            privileges,
        })
    }
}