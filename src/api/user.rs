// api/user.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use crate::{api::ConfigManager, AppState};

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub player_uuid: String,
    pub player_name: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub player_uuid: String,
    pub player_name: String,
    pub wallet: i64,
    pub bank: i64,
    pub is_bank_open: i8,
}

#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    pub from: String,    // "wallet" or "bank"
    pub to: String,      // "wallet" or "bank"  
    pub amount: i64,
}

#[derive(Debug, Serialize)]
pub struct TransferResponse {
    pub success: bool,
    pub message: String,
    pub new_wallet: i64,
    pub new_bank: i64,
    pub fee_charged: i64,
    pub amount_transferred: i64,
}

async fn insert_user(pool: &MySqlPool, user: User) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        "INSERT INTO tb_user (player_uuid, player_name) VALUES (?, ?)",
        user.player_uuid,
        user.player_name
    )
    .execute(pool)
    .await?;

    Ok(result.last_insert_id())
}

pub async fn get_user_by_uuid(pool: &MySqlPool, uuid: &str) -> Result<Option<UserResponse>, sqlx::Error> {
    let user = sqlx::query_as!(
        UserResponse,
        "SELECT id, player_uuid, player_name, wallet, bank, is_bank_open FROM tb_user WHERE player_uuid = ?",
        uuid
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

async fn get_wallet(pool: &MySqlPool, uuid: &str) -> Result<Option<i64>, sqlx::Error> {
    let record = sqlx::query!(
        "SELECT wallet FROM tb_user WHERE player_uuid = ?",
        uuid
    )
    .fetch_optional(pool)
    .await?;

    Ok(record.map(|r| r.wallet))
}
async fn get_bank(pool: &MySqlPool, uuid: &str) -> Result<Option<i64>, sqlx::Error> {
    let record = sqlx::query!(
        "SELECT bank FROM tb_user WHERE player_uuid = ?",
        uuid
    )
    .fetch_optional(pool)
    .await?;

    Ok(record.map(|r| r.bank))
}

async fn wallet_to_bank_with_fee(pool: &MySqlPool, uuid: &str, amount: i64) -> Result<(u64, i64), sqlx::Error> {
    let config = ConfigManager::load_from_db(pool).await?;
    let fee = config.calculate_transfer_fee("wallet", "bank", amount);
    let total_deducted = amount + fee;

    let result = sqlx::query!(
        "UPDATE tb_user SET wallet = wallet - ?, bank = bank + ? WHERE player_uuid = ? AND wallet >= ? AND is_bank_open = 1",
        total_deducted,
        amount,
        uuid,
        total_deducted
    )
    .execute(pool)
    .await?;

    Ok((result.rows_affected(), fee))
}

async fn bank_to_wallet_with_fee(pool: &MySqlPool, uuid: &str, amount: i64) -> Result<(u64, i64), sqlx::Error> {
    let config = ConfigManager::load_from_db(pool).await?;
    let fee = config.calculate_transfer_fee("bank", "wallet", amount);
    let total_deducted = amount + fee;

    let result = sqlx::query!(
        "UPDATE tb_user SET bank = bank - ?, wallet = wallet + ? WHERE player_uuid = ? AND bank >= ? AND is_bank_open = 1",
        total_deducted,
        amount,
        uuid,
        total_deducted
    )
    .execute(pool)
    .await?;

    Ok((result.rows_affected(), fee))
}

pub async fn create_user(
    State(pool): State<AppState>,
    Json(payload): Json<User>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match insert_user(&pool.pool, payload).await {
        Ok(user_id) => {
            tracing::info!("User created successfully with id: {}", user_id);
            Ok(Json(serde_json::json!({
                "success": true,
                "user_id": user_id,
                "message": "User created successfully"
            })))
        },
        Err(e) => {
            tracing::error!("Failed to create user: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
    }
}

pub async fn get_user(
    Path(uuid): Path<String>,
    State(pool): State<AppState>,
) -> Result<Json<UserResponse>, StatusCode> {
    match get_user_by_uuid(&pool.pool, &uuid).await {
        Ok(Some(user)) => {
            tracing::info!("Found user '{}' with UUID {}", user.player_name, uuid);
            Ok(Json(user))
        },
        Ok(None) => {
            tracing::warn!("No user found with UUID {}", uuid);
            Err(StatusCode::NOT_FOUND)
        },
        Err(e) => {
            tracing::error!("Database error while fetching user {}: {:?}", uuid, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
    }
}
pub async fn get_user_wallet(
    Path(uuid): Path<String>,
    State(pool): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match get_wallet(&pool.pool, &uuid).await {
        Ok(Some(wallet)) => Ok(Json(serde_json::json!({ "wallet": wallet }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Database error while fetching wallet for {}: {:?}", uuid, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
    }
}
pub async fn get_user_bank(
    Path(uuid): Path<String>,
    State(pool): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match get_bank(&pool.pool, &uuid).await {
        Ok(Some(bank)) => Ok(Json(serde_json::json!({ "bank": bank }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Database error while fetching bank for {}: {:?}", uuid, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
    }
}

pub async fn transfer_money(
    Path(uuid): Path<String>,
    State(pool): State<AppState>,
    Json(payload): Json<TransferRequest>,
) -> Result<Json<TransferResponse>, StatusCode> {
    let transfer_result = match (payload.from.as_str(), payload.to.as_str()) {
        ("wallet", "bank") => wallet_to_bank_with_fee(&pool.pool, &uuid, payload.amount).await,
        ("bank", "wallet") => bank_to_wallet_with_fee(&pool.pool, &uuid, payload.amount).await,
        _ => {
            return Ok(Json(TransferResponse {
                success: false,
                message: "Invalid transfer direction. Use 'wallet' or 'bank'".to_string(),
                new_wallet: 0,
                new_bank: 0,
                fee_charged: 0,
                amount_transferred: payload.amount,
            }));
        }
    };

    match transfer_result {
        Ok((rows_affected, fee)) => {
            if rows_affected == 0 {
                // Check if it's a bank access issue or insufficient funds
                let user = get_user_by_uuid(&pool.pool, &uuid).await.unwrap_or(None);
                let error_msg = match user {
                    Some(u) if u.is_bank_open == 0 => "Bank is not open! Visit a bank to access your account".to_string(),
                    Some(u) => {
                        let required = payload.amount + fee;
                        let available = if payload.from == "wallet" { u.wallet } else { u.bank };
                        format!("Insufficient funds in {} (have: {}, need: {})", payload.from, available, required)
                    }
                    None => "User not found".to_string(),
                };
                
                return Ok(Json(TransferResponse {
                    success: false,
                    message: error_msg,
                    new_wallet: 0,
                    new_bank: 0,
                    fee_charged: fee,
                    amount_transferred: payload.amount,
                }));
            }

            let user = match get_user_by_uuid(&pool.pool, &uuid).await {
                Ok(Some(user)) => user,
                _ => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            };

            Ok(Json(TransferResponse {
                success: true,
                message: format!("Transferred {} from {} to {} (fee: {})", payload.amount, payload.from, payload.to, fee),
                new_wallet: user.wallet,
                new_bank: user.bank,
                fee_charged: fee,
                amount_transferred: payload.amount,
            }))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}