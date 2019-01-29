use bigneon_db::prelude::*;
use config::Config;
use db::Connection;
use domain_events::executor_future::ExecutorFuture;
use domain_events::routing::DomainActionExecutor;
use errors::BigNeonError;
use futures::future;
use globee::GlobeeClient;
use globee::GlobeeIpnRequest;
use uuid::Uuid;

pub struct ProcessPaymentIPNExecutor {
    globee_api_key: String,
    globee_base_url: String,
}

impl DomainActionExecutor for ProcessPaymentIPNExecutor {
    fn execute(&self, action: DomainAction, conn: Connection) -> ExecutorFuture {
        match self.perform_job(&action, &conn) {
            Ok(_) => ExecutorFuture::new(action, conn, Box::new(future::ok(()))),
            Err(e) => ExecutorFuture::new(action, conn, Box::new(future::err(e))),
        }
    }
}

impl ProcessPaymentIPNExecutor {
    pub fn new(config: &Config) -> ProcessPaymentIPNExecutor {
        ProcessPaymentIPNExecutor {
            globee_api_key: config.globee_api_key.clone(),
            globee_base_url: config.globee_base_url.clone(),
        }
    }

    fn perform_job(&self, action: &DomainAction, conn: &Connection) -> Result<(), BigNeonError> {
        let ipn: GlobeeIpnRequest = serde_json::from_value(action.payload.clone())?;
        if ipn.custom_payment_id.is_none() {
            // TODO: Return failed?
            return Ok(());
        }
        let _client = GlobeeClient::new(self.globee_api_key.clone(), self.globee_base_url.clone());

        // TODO: Check with Globee server to verify that IPN is valid

        let order_id = Uuid::parse_str(ipn.custom_payment_id.as_ref().unwrap())?;
        let connection = conn.get();
        let mut order = Order::find(order_id, connection)?;

        let external_reference = format!("globee-{:?}", ipn.id);
        let status = match ipn
            .status
            .clone()
            .unwrap_or("none".to_string())
            .to_lowercase()
            .as_str()
        {
            "unpaid" => PaymentStatus::Unpaid,
            "paid" => PaymentStatus::PendingConfirmation,
            "overpaid" => PaymentStatus::PendingConfirmation,
            "underpaid" => PaymentStatus::PendingConfirmation,
            "paid_late" => PaymentStatus::PendingConfirmation,
            "confirmed" => PaymentStatus::Completed,
            "completed" => PaymentStatus::Completed,
            "refunded" => PaymentStatus::Refunded,
            "cancelled" => PaymentStatus::Cancelled,
            "draft" => PaymentStatus::Draft,
            _ => PaymentStatus::Unknown,
        };

        let payment =
            match Payment::find_by_order(order_id, &external_reference, connection).optional()? {
                Some(p) => p,
                None => order.add_provider_payment(
                    Some(external_reference.to_string()),
                    "globee".to_string(),
                    None,
                    (ipn.payment_details.received_amount.unwrap_or(0f64) * 100f64) as i64,
                    status,
                    action.payload.clone(),
                    connection,
                )?,
            };

        if status == PaymentStatus::Completed {
            payment.mark_complete(json!(ipn), None, connection)?;
        } else {
            payment.add_ipn(status, json!(ipn), None, connection)?;
        }

        Ok(())
    }
}
