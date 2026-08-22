use std::{collections::HashMap, sync::Arc};

use axum::Router;
use sqlx::PgPool;
use tokio::{net::TcpListener, sync::Mutex};
use tracing::info;
use tracing_subscriber::{
    Layer, fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt,
};

use crate::{models::Asset, routes};

#[derive(Clone)]
pub struct AppState {
    #[allow(dead_code)]
    pub assets: Arc<Mutex<HashMap<i64, Asset>>>,
    pub db: PgPool,
    pub jwt_secret: String,
}

impl AppState {
    async fn new() -> color_eyre::Result<Self> {
        let database_url = std::env::var("DATABASE_URL")?;
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "sua-chave-secreta-padrao-caso-nao-defina-no-env".to_string());

        let db = PgPool::connect(&database_url).await?;

        Ok(Self {
            assets: Default::default(),
            db,
            jwt_secret,
        })
    }
}

pub struct App;

impl App {
    pub async fn start() -> color_eyre::Result<()> {
        let layer = tracing_subscriber::fmt::layer()
            .with_span_events(FmtSpan::NEW)
            .boxed();

        tracing_subscriber::registry().with(layer).init();

        dotenvy::dotenv()?;

        let state = AppState::new().await?;

        let listener = TcpListener::bind("0.0.0.0:3000").await?;
        let router = Router::new()
            .nest("/api", routes::api::router())
            .merge(routes::frontend::router())
            .with_state(state);

        info!("Starting service");

        axum::serve(listener, router).await?;

        Ok(())
    }
}
