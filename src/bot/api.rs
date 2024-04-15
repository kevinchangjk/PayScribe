use reqwest;
use reqwest::header;
use serde_json::Value;
use std::error::Error;

/* API contains the logic for calling external APIs.
 * Links the bot's logic with anything it needs from the internet.
 * Called and used by the Processor only.
 */
async fn fetch_currencies(
    base_currency: &str,
    target_currency: &str,
) -> Result<f64, Box<dyn Error>> {
    let url = format!("https://cdn.jsdelivr.net/npm/@fawazahmed0/currency-api@latest/v1/currencies/{base_currency}.json");

    let mut h = header::HeaderMap::new();
    h.insert(
        "Accept",
        header::HeaderValue::from_static("application/json"),
    );

    let client = reqwest::Client::builder().default_headers(h).build()?;

    let response: Value = client.get(url).send().await?.json().await?;
    if let Some(conversions) = response.get(base_currency) {
        if let Some(value) = conversions.get(target_currency) {
            let res = value.as_f64();
            match res {
                Some(v) => return Ok(v),
                None => {
                    let res = value.as_i64();
                    if let Some(v) = res {
                        return Ok(v as f64);
                    }
                }
            }
        }
    }

    Err("Currency not found".into())
}

mod tests {
    use super::fetch_currencies;

    #[tokio::test]
    async fn test_fetch_currencies_api() {
        let fetch = fetch_currencies("usd", "eur").await;
        assert!(fetch.is_ok());
        assert!(fetch.unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_fetch_non_existent_currencies() {
        let fetch = fetch_currencies("usd", "non_existent_currency").await;
        assert!(fetch.is_err());
    }
}
