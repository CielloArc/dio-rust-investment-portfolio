use axum::extract::FromRequestParts;

use crate::{app::AppState, auth::user::User, error::AppError};

pub struct Admin(#[allow(dead_code)] pub User);

impl FromRequestParts<AppState> for Admin {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = User::from_request_parts(parts, state).await?;

        if user.is_admin() {
            Ok(Admin(user))
        } else {
            Err(AppError::InvalidCredentials)
        }
    }
}
