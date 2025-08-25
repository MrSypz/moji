// api/market.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use crate::{api::ConfigManager, AppState};

#[derive(Debug, Deserialize)]
pub struct SellItemRequest {
    pub item_key: String,
    pub quantity: i32,
}

#[derive(Debug, Serialize)]
pub struct SellItemResponse {
    pub success: bool,
    pub message: String,
    pub gross_earned: i64,
    pub transaction_fee: i64,
    pub vat: i64,
    pub net_earned: i64,
    pub price_per_unit: i64,
    pub new_wallet: i64,
    pub new_bank: i64,
    pub new_item_price: i64,
}

#[derive(Debug, Serialize)]
pub struct MarketItem {
    pub id: i32,
    pub item_key: String,
    pub item_name: String,
    pub base_price: i64,
    pub current_sell_price: i64,
    pub current_buy_price: i64,
    pub total_sold: i64,
    pub total_bought: i64,
    pub price_multiplier: f64,
}
#[derive(Debug, Serialize)]
pub struct LightMarketItem {
    pub item_key: String,
    pub current_sell_price: i64,
    pub price_multiplier: f64,
}

// POST /api/market/sell/{uuid} - Player sells items
pub async fn sell_item(
    Path(uuid): Path<String>,
    State(pool): State<AppState>,
    Json(payload): Json<SellItemRequest>,
) -> Result<Json<SellItemResponse>, StatusCode> {
    let config = match ConfigManager::load_from_db(&pool.pool).await {
        Ok(config) => config,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let market_item = match get_market_item(&pool.pool, &payload.item_key).await {
        Ok(Some(item)) => item,
        Ok(None) => {
            return Ok(Json(SellItemResponse {
                success: false,
                message: "Item not available in market".to_string(),
                gross_earned: 0,
                transaction_fee: 0,
                vat: 0,
                net_earned: 0,
                price_per_unit: 0,
                new_wallet: 0,
                new_bank: 0,
                new_item_price: 0,
            }));
        }
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let price_per_unit = market_item.current_sell_price;
    let gross_earned = price_per_unit * payload.quantity as i64;
    
    // Calculate fees using config
    let fees = config.calculate_market_fees(gross_earned);

    let mut tx = match pool.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Update player wallet only (no bank option)
    let update_result = sqlx::query!(
        "UPDATE tb_user SET wallet = wallet + ? WHERE player_uuid = ?",
        fees.net_amount,
        uuid
    )
    .execute(&mut *tx)
    .await;

    if update_result.is_err() {
        let _ = tx.rollback().await;
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Record transaction
    let transaction_result = sqlx::query!(
        "INSERT INTO tb_market_transactions (player_uuid, item_key, transaction_type, quantity, price_per_unit, total_amount, price_multiplier) VALUES (?, ?, 'SELL', ?, ?, ?, ?)",
        uuid,
        payload.item_key,
        payload.quantity,
        price_per_unit,
        fees.gross_amount,
        market_item.price_multiplier
    )
    .execute(&mut *tx)
    .await;

    if transaction_result.is_err() {
        let _ = tx.rollback().await;
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let market_update_result = sqlx::query!(
        "UPDATE tb_market_items SET total_sold = total_sold + ? WHERE item_key = ?",
        payload.quantity,
        payload.item_key
    )
    .execute(&mut *tx)
    .await;

    if market_update_result.is_err() {
        let _ = tx.rollback().await;
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    if tx.commit().await.is_err() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let new_price = update_market_price(&pool.pool, &payload.item_key, "SELL", payload.quantity).await
        .unwrap_or(price_per_unit);

    let user = match crate::api::user::get_user_by_uuid(&pool.pool, &uuid).await {
        Ok(Some(user)) => user,
        _ => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    Ok(Json(SellItemResponse {
        success: true,
        message: format!("Successfully sold {} x{}", payload.item_key, payload.quantity),
        gross_earned: fees.gross_amount,
        transaction_fee: fees.transaction_fee,
        vat: fees.vat,
        net_earned: fees.net_amount,
        price_per_unit,
        new_wallet: user.wallet,
        new_bank: user.bank,
        new_item_price: new_price,
    }))
}

// GET /api/market/items - Get all market items
pub async fn get_market_items(
    State(pool): State<AppState>,
) -> Result<Json<Vec<MarketItem>>, StatusCode> {
    match get_all_market_items(&pool.pool).await {
        Ok(items) => Ok(Json(items)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// GET /api/market/item/{key} - Get specific market item
pub async fn get_market_item_endpoint(
    Path(item_key): Path<String>,
    State(pool): State<AppState>,
) -> Result<Json<MarketItem>, StatusCode> {
    match get_market_item(&pool.pool, &item_key).await {
        Ok(Some(item)) => Ok(Json(item)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
// GET /api/market/item/light - Get specific market item price
pub async fn get_market_items_light(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<LightMarketItem>>, StatusCode> {
    match get_all_market_items_light(&app_state.pool).await {
        Ok(items) => Ok(Json(items)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_market_item(pool: &MySqlPool, item_key: &str) -> Result<Option<MarketItem>, sqlx::Error> {
    let item = sqlx::query_as!(
        MarketItem,
        "SELECT id, item_key, item_name, base_price, current_sell_price, current_buy_price, total_sold, total_bought, price_multiplier FROM tb_market_items WHERE item_key = ?",
        item_key
    )
    .fetch_optional(pool)
    .await?;

    Ok(item)
}
async fn get_all_market_items_light(pool: &MySqlPool) -> Result<Vec<LightMarketItem>, sqlx::Error> {
    let items = sqlx::query_as!(
        LightMarketItem,
        "SELECT item_key, current_sell_price, price_multiplier FROM tb_market_items ORDER BY item_key"
    )
    .fetch_all(pool)
    .await?;

    Ok(items)
}

async fn get_all_market_items(pool: &MySqlPool) -> Result<Vec<MarketItem>, sqlx::Error> {
    let items = sqlx::query_as!(
        MarketItem,
        "SELECT id, item_key, item_name, base_price, current_sell_price, current_buy_price, total_sold, total_bought, price_multiplier FROM tb_market_items ORDER BY item_name"
    )
    .fetch_all(pool)
    .await?;

    Ok(items)
}

async fn update_market_price(pool: &MySqlPool, item_key: &str, transaction_type: &str, quantity: i32) -> Result<i64, sqlx::Error> {
    let recent_sales = sqlx::query!(
        "SELECT CAST(COALESCE(SUM(quantity), 0) AS SIGNED) as total_sold FROM tb_market_transactions 
         WHERE item_key = ? AND transaction_type = 'SELL' AND timestamp >= DATE_SUB(NOW(), INTERVAL 1 HOUR)",
        item_key
    )
    .fetch_one(pool)
    .await?;

    let recent_buys = sqlx::query!(
        "SELECT CAST(COALESCE(SUM(quantity), 0) AS SIGNED) as total_bought FROM tb_market_transactions 
         WHERE item_key = ? AND transaction_type = 'BUY' AND timestamp >= DATE_SUB(NOW(), INTERVAL 1 HOUR)",
        item_key
    )
    .fetch_one(pool)
    .await?;

    let sales_volume = recent_sales.total_sold as f64;
    let buy_volume = recent_buys.total_bought as f64;

    let item = get_market_item(pool, item_key).await?.unwrap();
    let base_price = item.base_price as f64;
    let mut current_multiplier = item.price_multiplier;

    let supply_demand_ratio = if buy_volume > 0.0 {
        sales_volume / buy_volume
    } else if sales_volume > 0.0 {
        2.0
    } else {
        1.0
    };

    let price_change = match transaction_type {
        "SELL" => {
            let volume_factor = (quantity as f64) / 64.0;
            -0.02 * volume_factor * (1.0 + supply_demand_ratio * 0.5)
        }
        "BUY" => {
            let volume_factor = (quantity as f64) / 64.0;
            0.02 * volume_factor * (1.0 + (1.0 / supply_demand_ratio.max(0.1)) * 0.5)
        }
        _ => 0.0,
    };

    current_multiplier += price_change;
    let baseline_pull = (1.0 - current_multiplier) * 0.001;
    current_multiplier += baseline_pull;
    current_multiplier = current_multiplier.max(0.1).min(4.0);

    let new_sell_price = (base_price * current_multiplier) as i64;
    let new_buy_price = (base_price * current_multiplier * 1.6) as i64;

    sqlx::query!(
        "UPDATE tb_market_items SET current_sell_price = ?, current_buy_price = ?, price_multiplier = ? WHERE item_key = ?",
        new_sell_price,
        new_buy_price,
        current_multiplier,
        item_key
    )
    .execute(pool)
    .await?;

    tracing::info!(
        "Updated price for {}: multiplier {:.4} -> sell: {}, buy: {}",
        item_key, current_multiplier, new_sell_price, new_buy_price
    );

    Ok(new_sell_price)
}