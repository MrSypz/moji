mod api;
mod config;
mod services;

use std::net::SocketAddr;

use api::user::{create_user, get_user};
use axum::{
    Router,
    http::{HeaderName, Method},
    routing::{get, post},
};
use config::create_pool;
use sqlx::MySqlPool;
use tower_http::cors::CorsLayer;

use crate::{
    api::{
        market::{get_market_item_endpoint, get_market_items, get_market_items_light, sell_item}, user::{get_user_bank, get_user_wallet, transfer_money}, ConfigManager
    },
    services::price_regeneration::PriceRegenerationService,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: MySqlPool,
    pub config: ConfigManager,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let db_pool = match create_pool().await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!("Failed to connect to database: {}", e);
            std::process::exit(1);
        }
    };
    tracing::info!("✅ Successfully connected to database");

    let cors = CorsLayer::new()
        .allow_origin([
            "http://localhost:9696".parse().unwrap(),
            "http://127.0.0.1:9696".parse().unwrap(),
        ])
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([
            HeaderName::from_static("content-type"),
            HeaderName::from_static("authorization"),
        ])
        .allow_credentials(true);

    let addr = SocketAddr::from(([0, 0, 0, 0], 9696));
    tracing::info!("🚀 Server running at http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    let config = ConfigManager::load_from_db(&db_pool).await.unwrap();
    tracing::info!(
        "✅ Configuration loaded: VAT={}%, Transfer={}%, WalletToBank={}%, Threshold={}, MarketFee={}%",
        config.market_vat_rate * 100.0,
        config.transfer_fee_rate * 100.0,
        config.wallet_to_bank_fee_rate * 100.0,
        config.wallet_to_bank_threshold,
        config.market_transaction_fee * 100.0
    );

    let regen_service = PriceRegenerationService::new(db_pool.clone());
    tokio::spawn(async move {
        regen_service.start().await;
    });

    let app_state = AppState {
        pool: db_pool,
        config,
    };
    let app = Router::new()
        .route("/api/user", post(create_user))
        .route("/api/user/{uuid}", get(get_user))
        .route("/api/user/{uuid}/wallet", get(get_user_wallet))
        .route("/api/user/{uuid}/bank", get(get_user_bank))
        .route("/api/user/{uuid}/transfer", post(transfer_money))
        .route("/api/market/sell/{uuid}", post(sell_item))
        .route("/api/market/items", get(get_market_items))
        .route("/api/market/item/{key}", get(get_market_item_endpoint))
        .route("/api/market/items/light", get(get_market_items_light))
        .with_state(app_state)
        .layer(cors);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
