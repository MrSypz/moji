// services/price_regeneration.rs
use sqlx::MySqlPool;
use tokio::time::{interval, Duration};
use tracing;

pub struct PriceRegenerationService {
    pool: MySqlPool,
}

impl PriceRegenerationService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn start(&self) {
        let mut interval_timer = interval(Duration::from_secs(3 * 60 * 60)); // 3 hours
        
        tracing::info!("🔄 Price regeneration service started (every 3 hours)");
        
        loop {
            interval_timer.tick().await;
            
            if let Err(e) = self.regenerate_prices().await {
                tracing::error!("Price regeneration failed: {:?}", e);
            } else {
                tracing::info!("✅ Price regeneration completed");
            }
        }
    }

    async fn regenerate_prices(&self) -> Result<(), sqlx::Error> {
        let result = sqlx::query!(
            "UPDATE tb_market_items SET 
             price_multiplier = LEAST(3.0, price_multiplier + (1.0 - price_multiplier) * 0.1),
             current_sell_price = ROUND(base_price * LEAST(3.0, price_multiplier + (1.0 - price_multiplier) * 0.1)),
             current_buy_price = ROUND(base_price * LEAST(3.0, price_multiplier + (1.0 - price_multiplier) * 0.1) * 1.6)"
        )
        .execute(&self.pool)
        .await?;

        tracing::info!("Regenerated prices for {} items", result.rows_affected());
        Ok(())
    }
}