use axum::{
    Form, Json, Router,
    extract::Path,
    response::{IntoResponse, Redirect},
    routing::{get, patch, post},
};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;

use crate::{
    app::AppState,
    auth::{admin::Admin, user::User},
    error::AppError,
    models::Asset,
    repository::Repository,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/assets", get(list_assets).post(create_asset))
        .route("/assets/{id}", patch(update_asset))
        .route("/assets/create", post(create_asset_form))
        .route("/purchases/create", post(create_purchase_form))
        .route("/purchases/{id}/delete", post(delete_purchase_form))
}

#[derive(Deserialize)]
struct YahooResponse {
    chart: YahooChart,
}

#[derive(Deserialize)]
struct YahooChart {
    result: Option<Vec<YahooResult>>,
}

#[derive(Deserialize)]
struct YahooResult {
    meta: YahooMeta,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct YahooMeta {
    regularMarketPrice: f64,
}

pub async fn fetch_unit_value_in_usd(symbol: &str) -> Result<Decimal, Box<dyn std::error::Error>> {
    let symbol_upper = symbol.to_uppercase();

    let ticker = match symbol_upper.as_str() {
        "USD" => return Ok(Decimal::ONE),
        "EUR" => "EURUSD=X",
        "BRL" => "BRLUSD=X",
        "GLD" => "GLD",
        "BTC" | "BITCOIN" => "BTC-USD",
        "ETH" | "ETHEREUM" => "ETH-USD",
        other => other,
    };

    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1m&range=1d",
        ticker
    );

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0")
        .build()?;

    let res: YahooResponse = client.get(&url).send().await?.json().await?;

    if let Some(results) = res.chart.result {
        if let Some(first) = results.first() {
            let price_str = first.meta.regularMarketPrice.to_string();
            return Ok(Decimal::from_str(&price_str)?);
        }
    }

    Err("Failed to get the asset quote".into())
}

#[tracing::instrument(skip_all)]
pub async fn delete_purchase_form(
    Path(purchase_id): Path<i64>,
    user: User,
    repository: Repository,
) -> Result<impl IntoResponse, AppError> {
    repository.delete_purchase(purchase_id, user.id).await?;
    Ok(Redirect::to("/assets"))
}

#[tracing::instrument(skip_all)]
async fn list_assets(repository: Repository) -> Result<Json<Vec<Asset>>, AppError> {
    let assets = repository.list_assets().await?;
    Ok(Json(assets))
}

#[derive(Deserialize)]
struct CreateAssetRequest {
    name: String,
    unit_value: Decimal,
}

#[tracing::instrument(skip_all)]
async fn create_asset(
    _admin: Admin,
    repository: Repository,
    Json(request): Json<CreateAssetRequest>,
) -> Result<Json<Asset>, AppError> {
    let new_asset = repository
        .create_asset(request.name, request.unit_value)
        .await?;

    Ok(Json(new_asset))
}

#[derive(Deserialize)]
pub struct CreateAssetForm {
    pub name: String,
}

#[tracing::instrument(skip_all)]
pub async fn create_asset_form(
    _user: User,
    repository: Repository,
    Form(form): Form<CreateAssetForm>,
) -> Result<impl IntoResponse, AppError> {
    let unit_value = fetch_unit_value_in_usd(&form.name)
        .await
        .unwrap_or(Decimal::ONE);

    repository.create_asset(form.name, unit_value).await?;

    Ok(Redirect::to("/assets"))
}

#[derive(Deserialize)]
pub struct CreatePurchaseForm {
    pub asset_id: i64,
    pub quantity: Decimal,
}

#[tracing::instrument(skip_all)]
pub async fn create_purchase_form(
    user: User,
    repository: Repository,
    Form(form): Form<CreatePurchaseForm>,
) -> Result<impl IntoResponse, AppError> {
    let assets = repository.list_assets().await?;
    let asset = assets
        .into_iter()
        .find(|a| a.id == form.asset_id)
        .ok_or(AppError::AssetDoesNotExist)?;

    let current_price_usd = fetch_unit_value_in_usd(&asset.name)
        .await
        .unwrap_or(Decimal::ONE);

    repository.update_asset(asset.id, current_price_usd).await?;
    repository
        .insert_owned_asset(user.id, asset.id, form.quantity, current_price_usd)
        .await?;

    Ok(Redirect::to("/assets"))
}

#[derive(Deserialize)]
struct UpdateAssetRequest {
    unit_value: Decimal,
}

#[tracing::instrument(skip_all)]
async fn update_asset(
    Path(id): Path<i64>,
    _admin: Admin,
    repository: Repository,
    Json(request): Json<UpdateAssetRequest>,
) -> Result<Json<Asset>, AppError> {
    match repository.update_asset(id, request.unit_value).await? {
        Some(updated_asset) => Ok(Json(updated_asset)),
        None => Err(AppError::AssetDoesNotExist),
    }
}
