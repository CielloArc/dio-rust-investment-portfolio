use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;

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

#[derive(Deserialize)]
struct YahooMeta {
    regularMarketPrice: f64,
}

pub async fn fetch_unit_value_in_usd(symbol: &str) -> Result<Decimal, Box<dyn std::error::Error>> {
    let ticker = match symbol.to_uppercase().as_str() {
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
