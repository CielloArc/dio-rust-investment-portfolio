use std::convert::Infallible;

use axum::extract::FromRequestParts;
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::{
    app::AppState,
    error::AppError,
    models::{Asset, OwnedAsset},
};

#[derive(Debug, sqlx::FromRow)]
pub struct UserRecord {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub is_admin: bool,
}

pub struct Repository {
    db: PgPool,
}

#[allow(dead_code)]
impl Repository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn list_assets(&self) -> sqlx::Result<Vec<Asset>> {
        sqlx::query_as!(
            Asset,
            r#"
            SELECT id, name, unit_value
            FROM assets;
            "#
        )
        .fetch_all(&self.db)
        .await
    }

    pub async fn create_asset(&self, name: String, unit_value: Decimal) -> sqlx::Result<Asset> {
        sqlx::query_as!(
            Asset,
            r#"
            INSERT INTO assets (name, unit_value)
            VALUES ($1, $2)
            RETURNING id, name, unit_value;
            "#,
            name,
            unit_value
        )
        .fetch_one(&self.db)
        .await
    }

    pub async fn update_asset(
        &self,
        asset_id: i64,
        unit_value: Decimal,
    ) -> sqlx::Result<Option<Asset>> {
        sqlx::query_as!(
            Asset,
            r#"
            UPDATE assets
            SET unit_value = $2
            WHERE id = $1
            RETURNING id, name, unit_value;
            "#,
            asset_id,
            unit_value
        )
        .fetch_optional(&self.db)
        .await
    }

    pub async fn get_user_by_name(&self, username: &str) -> sqlx::Result<Option<UserRecord>> {
        sqlx::query_as!(
            UserRecord,
            r#"
            SELECT id, username, password_hash, is_admin
            FROM users
            WHERE username = $1;
            "#,
            username
        )
        .fetch_optional(&self.db)
        .await
    }

    pub async fn add_user(&self, username: &str, password_hash: &str) -> sqlx::Result<UserRecord> {
        sqlx::query_as!(
            UserRecord,
            r#"
            INSERT INTO users (username, password_hash, is_admin)
            VALUES ($1, $2, FALSE)
            RETURNING id, username, password_hash, is_admin;
            "#,
            username,
            password_hash
        )
        .fetch_one(&self.db)
        .await
    }

    pub async fn list_owned_assets(&self, user_id: i64) -> sqlx::Result<Vec<OwnedAsset>> {
        sqlx::query_as!(
            OwnedAsset,
            r#"
            SELECT
                a.id,
                a.name,
                a.unit_value,

                SUM(
                    (a.unit_value - o.bought_for)
                    * o.quantity_owned
                ) AS "value_delta!",

                SUM(o.quantity_owned) AS "quantity_owned!",

                JSON_AGG(
                    JSON_BUILD_OBJECT(
                        'id', o.id,
                        'bought_at', TO_CHAR(o.timestamp, 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"'),
                        'bought_for', o.bought_for,
                        'quantity_bought', o.quantity_owned,
                        'value_delta',
                            (a.unit_value - o.bought_for)
                            * o.quantity_owned
                    )
                    ORDER BY o.timestamp DESC
                ) AS "purchase_history!: _"

            FROM assets AS a

            JOIN owned_assets AS o
                ON o.asset_id = a.id

            WHERE o.user_id = $1

            GROUP BY
                a.id,
                a.name,
                a.unit_value;
            "#,
            user_id
        )
        .fetch_all(&self.db)
        .await
    }

    pub async fn insert_owned_asset(
        &self,
        user_id: i64,
        asset_id: i64,
        quantity: Decimal,
        unit_value: Decimal,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO owned_assets
                (user_id, asset_id, quantity_owned, bought_for)
            VALUES
                ($1, $2, $3, $4);
            "#,
            user_id,
            asset_id,
            quantity,
            unit_value,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn delete_purchase(&self, purchase_id: i64, user_id: i64) -> Result<(), AppError> {
        sqlx::query!(
            "DELETE FROM owned_assets WHERE id = $1 AND user_id = $2",
            purchase_id,
            user_id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
}

impl FromRequestParts<AppState> for Repository {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self {
            db: state.db.clone(),
        })
    }
}

#[cfg(test)]
impl From<PgPool> for Repository {
    fn from(db: PgPool) -> Self {
        Self { db }
    }
}
