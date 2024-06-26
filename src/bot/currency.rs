use reqwest;
use reqwest::header;
use serde_json::Value;
use std::error::Error;

// Represents a currency with a code and decimal places.
pub type Currency = (String, i32);

// Converts a (&str, i32) to a Currency.
fn to_currency(currency: (&str, i32)) -> Currency {
    (currency.0.to_string(), currency.1)
}

// Retrieves the currency given a currency code.
pub fn get_currency_from_code(code: &str) -> Option<Currency> {
    let code = code.to_uppercase();
    for currency in &CURRENCIES {
        if currency.0 == code {
            return Some(to_currency(*currency));
        }
    }

    None
}

pub fn get_default_currency() -> Currency {
    to_currency(CURRENCY_DEFAULT)
}

// Converts an amount from one currency to another, given the conversion rate.
pub fn convert_currency_with_rate(
    amount: i64,
    currency_from: &str,
    currency_to: &str,
    conversion_rate: f64,
) -> i64 {
    let currency_from = match get_currency_from_code(currency_from) {
        Some(currency) => currency,
        None => return amount,
    };
    let currency_to = match get_currency_from_code(currency_to) {
        Some(currency) => currency,
        None => return amount,
    };

    let result = amount as f64 * 10.0_f64.powi(currency_to.1 - currency_from.1) * conversion_rate;

    result.round() as i64
}

// Main API method that fetches currency conversions
pub async fn fetch_currency_conversion(
    base_currency: &str,
    target_currency: &str,
) -> Result<f64, Box<dyn Error>> {
    let base_currency = base_currency.to_lowercase();
    let target_currency = target_currency.to_lowercase();
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

// List of all supported currencies
pub const CURRENCIES: [(&str, i32); 167] = [
    ("AED", 2),
    ("AFN", 2),
    ("ALL", 2),
    ("AMD", 2),
    ("ANG", 2),
    ("AOA", 2),
    ("ARS", 2),
    ("AUD", 2),
    ("AWG", 2),
    ("AZN", 2),
    ("BAM", 2),
    ("BBD", 2),
    ("BDT", 2),
    ("BGN", 2),
    ("BHD", 3),
    ("BIF", 0),
    ("BMD", 2),
    ("BND", 2),
    ("BOB", 2),
    ("BOV", 2),
    ("BRL", 2),
    ("BSD", 2),
    ("BTN", 2),
    ("BWP", 2),
    ("BYN", 2),
    ("BZD", 2),
    ("CAD", 2),
    ("CDF", 2),
    ("CHE", 2),
    ("CHF", 2),
    ("CHW", 2),
    ("CLF", 4),
    ("CLP", 0),
    ("CNY", 2),
    ("COP", 2),
    ("COU", 2),
    ("CRC", 2),
    ("CUP", 2),
    ("CVE", 2),
    ("CZK", 2),
    ("DJF", 0),
    ("DKK", 2),
    ("DOP", 2),
    ("DZD", 2),
    ("EGP", 2),
    ("ERN", 2),
    ("ETB", 2),
    ("EUR", 2),
    ("FJD", 2),
    ("FKP", 2),
    ("GBP", 2),
    ("GEL", 2),
    ("GHS", 2),
    ("GIP", 2),
    ("GMD", 2),
    ("GNF", 0),
    ("GTQ", 2),
    ("GYD", 2),
    ("HKD", 2),
    ("HNL", 2),
    ("HTG", 2),
    ("HUF", 2),
    ("IDR", 2),
    ("ILS", 2),
    ("INR", 2),
    ("IQD", 3),
    ("IRR", 2),
    ("ISK", 0),
    ("JMD", 2),
    ("JOD", 3),
    ("JPY", 0),
    ("KES", 2),
    ("KGS", 2),
    ("KHR", 2),
    ("KMF", 0),
    ("KPW", 2),
    ("KRW", 0),
    ("KWD", 3),
    ("KYD", 2),
    ("KZT", 2),
    ("LAK", 2),
    ("LBP", 2),
    ("LKR", 2),
    ("LRD", 2),
    ("LSL", 2),
    ("LYD", 3),
    ("MAD", 2),
    ("MDL", 2),
    ("MGA", 2),
    ("MKD", 2),
    ("MMK", 2),
    ("MNT", 2),
    ("MOP", 2),
    ("MRU", 2),
    ("MUR", 2),
    ("MVR", 2),
    ("MWK", 2),
    ("MXN", 2),
    ("MXV", 2),
    ("MYR", 2),
    ("MZN", 2),
    ("NAD", 2),
    ("NGN", 2),
    ("NIO", 2),
    ("NOK", 2),
    ("NPR", 2),
    ("NZD", 2),
    ("OMR", 3),
    ("PAB", 2),
    ("PEN", 2),
    ("PGK", 2),
    ("PHP", 2),
    ("PKR", 2),
    ("PLN", 2),
    ("PYG", 0),
    ("QAR", 2),
    ("RON", 2),
    ("RSD", 2),
    ("RUB", 2),
    ("RWF", 0),
    ("SAR", 2),
    ("SBD", 2),
    ("SCR", 2),
    ("SDG", 2),
    ("SEK", 2),
    ("SGD", 2),
    ("SHP", 2),
    ("SLE", 2),
    ("SLL", 2),
    ("SOS", 2),
    ("SRD", 2),
    ("SSP", 2),
    ("STN", 2),
    ("SVC", 2),
    ("SYP", 2),
    ("SZL", 2),
    ("THB", 2),
    ("TJS", 2),
    ("TMT", 2),
    ("TND", 3),
    ("TOP", 2),
    ("TRY", 2),
    ("TTD", 2),
    ("TWD", 2),
    ("TZS", 2),
    ("UAH", 2),
    ("UGX", 0),
    ("USD", 2),
    ("USN", 2),
    ("UYI", 0),
    ("UYU", 2),
    ("UYW", 4),
    ("UZS", 2),
    ("VED", 2),
    ("VES", 2),
    ("VND", 0),
    ("VUV", 0),
    ("WST", 2),
    ("XAF", 0),
    ("XCD", 2),
    ("XOF", 0),
    ("XPF", 0),
    ("YER", 2),
    ("ZAR", 2),
    ("ZMW", 2),
    ("ZWL", 2),
    ("NIL", 2),
];
pub const CURRENCY_DEFAULT: (&str, i32) = ("NIL", 2);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_currencies_api() {
        let fetch = fetch_currency_conversion("usd", "eur").await;
        assert!(fetch.is_ok());
        assert!(fetch.unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_fetch_non_existent_currencies() {
        let fetch = fetch_currency_conversion("usd", "non_existent_currency").await;
        assert!(fetch.is_err());
    }
}
