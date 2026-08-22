use axum::extract::FromRequestParts;
use axum_extra::extract::CookieJar;
use jwt_simple::prelude::*;
use password_auth::VerifyError;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

use crate::{app::AppState, error::AppError, repository::Repository};

pub struct UnauthenticatedUser {
    username: String,
    password: String,
}

impl UnauthenticatedUser {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    pub async fn authenticate(&self, repository: &Repository) -> Result<User, AppError> {
        let user_record = match repository.get_user_by_name(&self.username).await? {
            Some(user_record) => user_record,
            None => return Err(AppError::UserDoesNotExist),
        };

        match password_auth::verify_password(&self.password, &user_record.password_hash) {
            Ok(()) => Ok(User::new(
                user_record.id,
                user_record.username,
                user_record.is_admin,
            )),
            Err(VerifyError::PasswordInvalid) => Err(AppError::InvalidCredentials),
            Err(VerifyError::Parse(err)) => panic!("Hashing algorithm failed: {err}"),
        }
    }

    pub async fn register(self, repository: &Repository) -> Result<User, AppError> {
        let password_hash = password_auth::generate_hash(self.password);
        let user_record = match repository.add_user(&self.username, &password_hash).await {
            Ok(user_record) => user_record,
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                return Err(AppError::UsernameTaken);
            }
            Err(err) => return Err(AppError::Database(err)),
        };

        Ok(User::new(user_record.id, user_record.username, false))
    }
}

#[derive(Clone, Debug)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub is_admin: bool,
}

impl User {
    pub fn new(id: i64, username: String, is_admin: bool) -> Self {
        Self {
            id,
            username,
            is_admin,
        }
    }

    pub fn username(&self) -> &String {
        &self.username
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn is_admin(&self) -> bool {
        self.is_admin
    }

    pub fn auth_token(&self, secret_key: &[u8]) -> Result<String, AppError> {
        let key = HS256Key::from_bytes(secret_key);
        let claims =
            Claims::with_custom_claims(UserClaim::from(self.clone()), Duration::from_mins(60));
        let token = key.authenticate(claims)?;
        Ok(token)
    }

    pub fn from_auth_token(token: &str, secret_key: &[u8]) -> Result<Self, AppError> {
        let key = HS256Key::from_bytes(secret_key);
        let claims: UserClaim = key.verify_token(token, None)?.custom;
        Ok(Self::new(claims.id, claims.username, claims.is_admin))
    }
}

impl FromRequestParts<AppState> for User {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);

        let token = match jar.get("token") {
            Some(token) => token.value(),
            None => return Err(AppError::MissingAuthorization),
        };

        User::from_auth_token(token, state.jwt_secret.as_bytes())
    }
}

pub struct OptionalUser(pub Option<User>);

impl FromRequestParts<AppState> for OptionalUser {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(OptionalUser(
            User::from_request_parts(parts, state).await.ok(),
        ))
    }
}

#[derive(Serialize, Deserialize)]
struct UserClaim {
    id: i64,
    username: String,
    is_admin: bool,
}

impl From<User> for UserClaim {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            is_admin: user.is_admin,
        }
    }
}
