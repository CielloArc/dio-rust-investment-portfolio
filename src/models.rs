use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;

#[derive(Serialize, Clone)]
pub struct Asset {
    pub id: i64,
    pub name: String,
    pub unit_value: Decimal,
}

#[allow(dead_code)]
pub struct UserRecord {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PurchaseHistory {
    pub id: i64,
    pub bought_at: String,
    pub quantity_bought: Decimal,
    pub bought_for: Decimal,
    pub value_delta: Decimal,
}

#[derive(Serialize)]
pub struct OwnedAsset {
    pub id: i64,
    pub name: String,
    pub unit_value: Decimal,
    pub value_delta: Decimal,
    pub quantity_owned: Decimal,
    pub purchase_history: Json<Vec<PurchaseHistory>>,
}
