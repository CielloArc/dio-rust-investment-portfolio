use askama::Template;
use axum::{
    Router,
    extract::{Form, State},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use rust_decimal::Decimal;
use serde::Deserialize;
use tokio::try_join;

use crate::{
    app::AppState,
    auth::{
        admin::Admin,
        user::{OptionalUser, UnauthenticatedUser, User},
    },
    error::AppError,
    models::{Asset, OwnedAsset},
    repository::Repository,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/login", get(login_page).post(login))
        .route("/logout", get(logout))
        .route("/assets", get(assets).post(purchase_asset))
        .route("/admin/assets/create", post(create_asset))
        .route("/admin/assets/update", post(update_asset))
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPage;

#[tracing::instrument(skip_all)]
async fn login_page() -> Result<Html<String>, AppError> {
    let html = LoginPage.render()?;
    Ok(Html(html))
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

async fn login(
    State(state): State<AppState>,
    repository: Repository,
    jar: CookieJar,
    Form(request): Form<LoginForm>,
) -> Result<impl IntoResponse, AppError> {
    let unauth_user = UnauthenticatedUser::new(request.username, request.password);

    let user = match unauth_user.authenticate(&repository).await {
        Ok(user) => user,
        Err(AppError::UserDoesNotExist) => unauth_user.register(&repository).await?,
        Err(other_err) => return Err(other_err),
    };

    let token = user.auth_token(state.jwt_secret.as_bytes())?;

    let cookie = Cookie::build(("token", token))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .build();

    Ok((jar.add(cookie), Redirect::to("/assets")))
}

pub async fn logout(jar: CookieJar) -> impl IntoResponse {
    (jar.remove(Cookie::from("token")), Redirect::to("/login"))
}

async fn index(maybe_user: OptionalUser) -> Result<Redirect, AppError> {
    match maybe_user.0 {
        Some(_) => Ok(Redirect::to("/assets")),
        None => Ok(Redirect::to("/login")),
    }
}

pub struct AssetShare {
    pub name: String,
    pub percentage: Decimal,
    pub accumulated_percentage: Decimal,
    pub color: String,
}

#[derive(Template)]
#[template(path = "assets.html")]
pub struct AssetsPage {
    pub owned_assets: Vec<OwnedAsset>,
    pub available_assets: Vec<Asset>,
    pub user: User,
    pub total_value: Decimal,
    pub total_delta: Decimal,
    pub distribution: Vec<AssetShare>,
}

impl AssetsPage {
    pub fn conic_gradient(&self) -> String {
        let mut stops = Vec::new();
        for share in &self.distribution {
            stops.push(format!(
                "{} 0 {}%",
                share.color, share.accumulated_percentage
            ));
        }
        format!("conic-gradient({})", stops.join(", "))
    }

    pub fn is_total_value_positive(&self) -> bool {
        self.total_value > Decimal::ZERO
    }

    pub fn is_total_delta_positive(&self) -> bool {
        self.total_delta >= Decimal::ZERO
    }

    pub fn is_positive(val: &Decimal) -> bool {
        *val >= Decimal::ZERO
    }
}

pub async fn assets(repository: Repository, user: User) -> Result<Html<String>, AppError> {
    let (owned_assets, available_assets) = try_join!(
        repository.list_owned_assets(user.id()),
        repository.list_assets()
    )?;

    let total_value: Decimal = owned_assets
        .iter()
        .map(|a| a.unit_value * a.quantity_owned)
        .sum();

    let total_delta: Decimal = owned_assets.iter().map(|a| a.value_delta).sum();

    let mut current_acc = Decimal::ZERO;
    let mut distribution = Vec::new();

    for (index, asset) in owned_assets.iter().enumerate() {
        let asset_total = asset.unit_value * asset.quantity_owned;
        let percentage = if total_value > Decimal::ZERO {
            ((asset_total / total_value) * Decimal::from(100)).round_dp(2)
        } else {
            Decimal::ZERO
        };

        current_acc += percentage;
        let color = format!("hsl({}, 80%, 60%)", (index + 1) * 60);

        distribution.push(AssetShare {
            name: asset.name.clone(),
            percentage,
            accumulated_percentage: current_acc,
            color,
        });
    }

    let html = AssetsPage {
        owned_assets,
        available_assets,
        user,
        total_value: total_value.round_dp(2),
        total_delta: total_delta.round_dp(2),
        distribution,
    }
    .render()?;

    Ok(Html(html))
}

#[derive(Deserialize)]
pub struct PurchaseAssetForm {
    pub asset_id: i64,
    pub unit_value: Decimal,
    pub quantity: Decimal,
}

pub async fn purchase_asset(
    repository: Repository,
    user: User,
    Form(request): Form<PurchaseAssetForm>,
) -> Result<Redirect, AppError> {
    repository
        .insert_owned_asset(
            user.id(),
            request.asset_id,
            request.quantity,
            request.unit_value,
        )
        .await?;

    Ok(Redirect::to("/assets"))
}

#[derive(Deserialize)]
pub struct CreateAssetForm {
    pub name: String,
    pub unit_value: Decimal,
}

pub async fn create_asset(
    _admin: Admin,
    repository: Repository,
    Form(request): Form<CreateAssetForm>,
) -> Result<Redirect, AppError> {
    repository
        .create_asset(request.name, request.unit_value)
        .await?;

    Ok(Redirect::to("/assets"))
}

#[derive(Deserialize)]
pub struct UpdateAssetForm {
    pub asset_id: i64,
    pub unit_value: Decimal,
}

pub async fn update_asset(
    _admin: Admin,
    repository: Repository,
    Form(request): Form<UpdateAssetForm>,
) -> Result<Redirect, AppError> {
    repository
        .update_asset(request.asset_id, request.unit_value)
        .await?;

    Ok(Redirect::to("/assets"))
}

pub mod filters {
    #[askama::filter_fn]
    pub fn human_datetime(
        datetime: &impl std::fmt::Display,
        _env: &dyn askama::Values,
    ) -> askama::Result<String> {
        Ok(datetime.to_string())
    }
}
