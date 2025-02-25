use anyhow::{anyhow, Result};
use rig_tool_macro::tool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Candlestick {
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopToken {
    pub name: String,
    pub pubkey: String,
    pub price: f64,
    pub market_cap: f64,
    pub volume_24h: f64,
    pub price_change_24h: f64,
}

const API_BASE: &str = "https://api.listen-rs.com/v1/adapter";

#[tool(description = "
Fetch top tokens from the Listen API.

Parameters:
- limit: Optional number of tokens to return (default: 20)
- min_volume: Optional minimum 24h volume filter
- min_market_cap: Optional minimum market cap filter
- timeframe: Optional timeframe in seconds
- only_pumpfun_tokens: Optional boolean to filter only PumpFun tokens (default: true)

Returns a list of top tokens with their market data.
")]
pub async fn fetch_top_tokens(
    limit: Option<usize>,
    min_volume: Option<f64>,
    min_market_cap: Option<f64>,
    timeframe: Option<u64>,
    only_pumpfun_tokens: Option<bool>,
) -> Result<Vec<TopToken>> {
    let mut url = format!("{}/top-tokens", API_BASE);
    let mut query_params = vec![];

    if let Some(limit) = limit {
        query_params.push(format!("limit={}", limit));
    }
    if let Some(min_volume) = min_volume {
        query_params.push(format!("min_volume={}", min_volume));
    }
    if let Some(min_market_cap) = min_market_cap {
        query_params.push(format!("min_market_cap={}", min_market_cap));
    }
    if let Some(timeframe) = timeframe {
        query_params.push(format!("timeframe={}", timeframe));
    }
    if let Some(only_pumpfun) = only_pumpfun_tokens {
        query_params.push(format!("only_pumpfun_tokens={}", only_pumpfun));
    }

    if !query_params.is_empty() {
        url = format!("{}?{}", url, query_params.join("&"));
    }

    let response = reqwest::get(&url)
        .await
        .map_err(|e| anyhow!("Failed to fetch top tokens: {}", e))?;

    let tokens = response
        .json::<Vec<TopToken>>()
        .await
        .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

    Ok(tokens)
}

#[tool(description = "
Fetch candlestick data for a token from the Listen API.

Parameters:
- mint: The token's mint/pubkey address
- interval: The candlestick interval, one of:
  * '15s' (15 seconds)
  * '30s' (30 seconds)
  * '1m'  (1 minute)
  * '5m'  (5 minutes)
  * '15m' (15 minutes)
  * '30m' (30 minutes)
  * '1h'  (1 hour)
  * '4h'  (4 hours)
  * '1d'  (1 day)
- limit: Optional number of candlesticks to return

for tokens under 1M market cap, use the 30s interval, 200 limit

for tokens over 1M market cap, use the 5m interval, 200 limit

for tokens over 10M market cap, use the 15m interval, 200 limit

Returns a list of candlesticks with OHLCV data.
")]
pub async fn fetch_candlesticks(
    mint: String,
    interval: String,
    limit: Option<usize>,
) -> Result<Vec<Candlestick>> {
    // Validate interval
    match interval.as_str() {
        "15s" | "30s" | "1m" | "5m" | "15m" | "30m" | "1h" | "4h" | "1d" => {
            ()
        }
        _ => return Err(anyhow!("Invalid interval: {}", interval)),
    }

    let mut url = format!(
        "{}/candlesticks?mint={}&interval={}",
        API_BASE, mint, interval
    );

    if let Some(limit) = limit {
        url = format!("{}&limit={}", url, limit);
    }

    let response = reqwest::get(&url)
        .await
        .map_err(|e| anyhow!("Failed to fetch candlesticks: {}", e))?;

    let candlesticks = response
        .json::<Vec<Candlestick>>()
        .await
        .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

    Ok(candlesticks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_top_tokens() {
        let tokens =
            fetch_top_tokens(Some(10), None, None, None, Some(true)).await;
        println!("{:?}", tokens);
    }

    #[tokio::test]
    async fn test_fetch_candlesticks() {
        let candlesticks = fetch_candlesticks(
            "So11111111111111111111111111111111111111112".to_string(),
            "5m".to_string(),
            Some(10),
        )
        .await;
        println!("{:?}", candlesticks);
    }
}
