// api/config.rs
use sqlx::MySqlPool;
use std::collections::HashMap;
use bigdecimal::ToPrimitive;
#[derive(Clone)]
pub struct ConfigManager {
    pub market_vat_rate: f64,
    pub transfer_fee_rate: f64,
    pub wallet_to_bank_fee_rate: f64,
    pub wallet_to_bank_threshold: i64,
    pub market_transaction_fee: f64,
}


#[derive(Debug)]
pub struct MarketFees {
    pub gross_amount: i64,
    pub transaction_fee: i64,
    pub vat: i64,
    pub net_amount: i64,
}


impl ConfigManager {
    pub async fn load_from_db(pool: &MySqlPool) -> Result<Self, sqlx::Error> {
        let configs = sqlx::query!(
            "SELECT config_key, config_value FROM tb_config"
        )
        .fetch_all(pool)
        .await?;

        let mut config_map: HashMap<String, f64> = HashMap::new();
        for config in configs {
            config_map.insert(config.config_key, config.config_value.to_f64().unwrap_or(0.0));
        }

        Ok(ConfigManager {
            market_vat_rate: *config_map.get("market_vat_rate").unwrap_or(&0.34),
            transfer_fee_rate: *config_map.get("transfer_fee_rate").unwrap_or(&0.10),
            wallet_to_bank_fee_rate: *config_map.get("wallet_to_bank_fee_rate").unwrap_or(&0.05),
            wallet_to_bank_threshold: *config_map.get("wallet_to_bank_threshold").unwrap_or(&10000.0) as i64,
            market_transaction_fee: *config_map.get("market_transaction_fee").unwrap_or(&0.02),
        })
    }

    pub fn calculate_transfer_fee(&self, from: &str, to: &str, amount: i64) -> i64 {
        match (from, to) {
            ("wallet", "bank") if amount >= self.wallet_to_bank_threshold => {
                (amount as f64 * self.wallet_to_bank_fee_rate) as i64
            }
            ("wallet", "bank") | ("bank", "wallet") => {
                (amount as f64 * self.transfer_fee_rate) as i64
            }
            _ => 0,
        }
    }

    pub fn calculate_market_fees(&self, gross_amount: i64) -> MarketFees {
        let transaction_fee = (gross_amount as f64 * self.market_transaction_fee) as i64;
        let taxable_amount = gross_amount - transaction_fee;
        let vat = (taxable_amount as f64 * self.market_vat_rate) as i64;
        let net_amount = gross_amount - transaction_fee - vat;

        MarketFees {
            gross_amount,
            transaction_fee,
            vat,
            net_amount,
        }
    }
}
