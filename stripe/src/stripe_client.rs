use reqwest;
use serde_json;
use ChargeResult;
use RefundResult;
use StripeError;

pub struct StripeClient {
    api_key: String,
}

impl StripeClient {
    pub fn new(api_key: String) -> StripeClient {
        StripeClient { api_key }
    }

    pub fn charge(
        &self,
        token: &str,
        amount: i64,
        currency: &str,
        description: &str,
        metadata: Vec<(String, String)>,
    ) -> Result<ChargeResult, StripeError> {
        self.create_charge(token, amount, currency, description, true, metadata)
    }

    pub fn auth(
        &self,
        token: &str,
        amount: i64,
        currency: &str,
        description: &str,
        metadata: Vec<(String, String)>,
    ) -> Result<ChargeResult, StripeError> {
        self.create_charge(token, amount, currency, description, false, metadata)
    }

    fn create_charge(
        &self,
        token: &str,
        amount: i64,
        currency: &str,
        description: &str,
        capture: bool,
        metadata: Vec<(String, String)>,
    ) -> Result<ChargeResult, StripeError> {
        let mut params = vec![
            ("currency".to_string(), currency.to_string()),
            ("amount".to_string(), amount.to_string()),
            ("description".to_string(), description.to_string()),
            ("source".to_string(), token.to_string()),
            ("capture".to_string(), capture.to_string()),
        ];

        for key_value in metadata {
            params.push((format!("metadata[{}]", key_value.0), key_value.1));
        }
        let client = reqwest::Client::new();
        let mut resp = client
            .post("https://api.stripe.com/v1/charges")
            .basic_auth(&self.api_key, Some(""))
            .form(&params)
            .send()?;
        match resp.status() {
            reqwest::StatusCode::OK => {
                return ChargeResult::from_response(resp);
            }
            _ => return Err(StripeError::from_response(&mut resp)),
        }
    }

    pub fn refund(&self, charge_id: &str) -> Result<RefundResult, StripeError> {
        let mut params = vec![("charge".to_string(), charge_id.to_string())];

        let client = reqwest::Client::new();
        let mut resp = client
            .post("https://api.stripe.com/v1/refunds")
            .basic_auth(&self.api_key, Some(""))
            .form(&params)
            .send()?;
        match resp.status() {
            reqwest::StatusCode::OK => {
                return RefundResult::from_response(resp);
            }
            _ => return Err(StripeError::from_response(&mut resp)),
        }
    }

    pub fn complete(&self, charge_id: &str) -> Result<ChargeResult, StripeError> {
        let client = reqwest::Client::new();

        let mut resp = client
            .post(&format!(
                "https://api.stripe.com/v1/charges/{}/capture",
                charge_id
            )).basic_auth(&self.api_key, Some(""))
            .send()?;
        match resp.status() {
            reqwest::StatusCode::OK => {
                return ChargeResult::from_response(resp);
            }
            _ => return Err(StripeError::from_response(&mut resp)),
        }
    }
}
