use chrono::prelude::*;
use diesel;
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Nullable, Text, Timestamp, Uuid as dUuid};
use itertools::Itertools;
use models::*;
use std::collections::HashMap;
use utils::errors::*;
use uuid::Uuid;

pub struct Report {}

#[derive(Serialize, Deserialize, PartialEq, Queryable, QueryableByName)]
pub struct TransactionReportRow {
    #[sql_type = "Text"]
    pub event_name: String,
    #[sql_type = "Text"]
    pub ticket_name: String,
    #[sql_type = "BigInt"]
    pub quantity: i64,
    #[sql_type = "BigInt"]
    pub refunded_quantity: i64,
    #[sql_type = "BigInt"]
    pub actual_quantity: i64,
    #[sql_type = "BigInt"]
    pub unit_price_in_cents: i64,
    #[sql_type = "BigInt"]
    pub gross: i64,
    #[sql_type = "BigInt"]
    pub company_fee_in_cents: i64,
    #[sql_type = "BigInt"]
    pub client_fee_in_cents: i64,
    #[sql_type = "BigInt"]
    pub gross_fee_in_cents: i64,
    #[sql_type = "BigInt"]
    pub gross_fee_in_cents_total: i64,
    #[sql_type = "BigInt"]
    pub event_fee_company_in_cents: i64,
    #[sql_type = "BigInt"]
    pub event_fee_client_in_cents: i64,
    #[sql_type = "BigInt"]
    pub event_fee_gross_in_cents: i64,
    #[sql_type = "BigInt"]
    pub event_fee_gross_in_cents_total: i64,
    #[sql_type = "Text"]
    pub order_type: OrderTypes,
    #[sql_type = "Nullable<Text>"]
    pub payment_method: Option<PaymentMethods>,
    #[sql_type = "Timestamp"]
    pub transaction_date: NaiveDateTime,
    #[sql_type = "Nullable<Text>"]
    pub redemption_code: Option<String>,
    #[sql_type = "dUuid"]
    pub order_id: Uuid,
    #[sql_type = "dUuid"]
    pub event_id: Uuid,
    #[sql_type = "dUuid"]
    pub user_id: Uuid,
    #[sql_type = "Text"]
    pub user_name: String,
    #[sql_type = "Text"]
    pub email: String,
}

#[derive(Serialize, Deserialize)]
pub struct EventSummarySalesResult {
    pub event_id: Uuid,
    pub sales: Vec<EventSummarySalesRow>,
    pub ticket_fees: Vec<EventSummaryFeesRow>,
    pub other_fees: Vec<EventSummaryOtherFees>,
}

impl Default for EventSummarySalesResult {
    fn default() -> Self {
        EventSummarySalesResult {
            event_id: Uuid::nil(),
            sales: vec![],
            ticket_fees: vec![],
            other_fees: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Queryable, QueryableByName)]
pub struct EventSummarySalesRow {
    #[sql_type = "dUuid"]
    pub event_id: Uuid,
    #[sql_type = "Text"]
    pub ticket_name: String,
    #[sql_type = "Text"]
    pub pricing_name: String,
    #[sql_type = "BigInt"]
    pub total_client_fee_in_cents: i64,
    #[sql_type = "BigInt"]
    pub total_company_fee_in_cents: i64,
    #[sql_type = "BigInt"]
    pub price_in_cents: i64,
    #[sql_type = "BigInt"]
    pub online_count: i64,
    #[sql_type = "BigInt"]
    pub box_office_count: i64,
    #[sql_type = "BigInt"]
    pub comp_count: i64,
    #[sql_type = "BigInt"]
    pub total_sold: i64,
    #[sql_type = "BigInt"]
    pub total_gross_income_in_cents: i64,
    #[sql_type = "dUuid"]
    pub ticket_type_id: Uuid,
    #[sql_type = "dUuid"]
    pub ticket_pricing_id: Uuid,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Queryable, QueryableByName)]
pub struct EventSummaryFeesRow {
    #[sql_type = "dUuid"]
    pub event_id: Uuid,

    #[sql_type = "dUuid"]
    pub ticket_type_id: Uuid,
    #[sql_type = "dUuid"]
    pub ticket_pricing_id: Uuid,
    #[sql_type = "Text"]
    pub ticket_name: String,
    #[sql_type = "Text"]
    pub pricing_name: String,
    #[sql_type = "BigInt"]
    pub total_sold: i64,
    #[sql_type = "BigInt"]
    pub comp_count: i64,
    #[sql_type = "BigInt"]
    pub online_count: i64,
    #[sql_type = "BigInt"]
    pub price_in_cents: i64,
    #[sql_type = "BigInt"]
    pub total_company_fee_in_cents: i64,
    #[sql_type = "BigInt"]
    pub company_fee_in_cents: i64,
    #[sql_type = "BigInt"]
    pub total_client_fee_in_cents: i64,
    #[sql_type = "BigInt"]
    pub client_fee_in_cents: i64,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Queryable, QueryableByName)]
pub struct EventSummaryOtherFees {
    #[sql_type = "dUuid"]
    pub event_id: Uuid,

    #[sql_type = "BigInt"]
    pub unit_price_in_cents: i64,
    #[sql_type = "BigInt"]
    pub total_company_fee_in_cents: i64,
    #[sql_type = "BigInt"]
    pub company_fee_in_cents: i64,
    #[sql_type = "BigInt"]
    pub total_client_fee_in_cents: i64,
    #[sql_type = "BigInt"]
    pub client_fee_in_cents: i64,
}

#[derive(Serialize, Deserialize)]
pub struct TicketCountResult {
    pub sales: Option<TicketCountSalesRow>,
    pub totals: Option<TicketCountRow>,
}

#[derive(Serialize, Deserialize, PartialEq, Queryable, QueryableByName)]
pub struct TicketCountRow {
    #[sql_type = "dUuid"]
    pub ticket_type_id: Uuid,
    #[sql_type = "Text"]
    pub ticket_name: String,
    #[sql_type = "Text"]
    pub ticket_stats: String,
    #[sql_type = "dUuid"]
    pub event_id: Uuid,
    #[sql_type = "Text"]
    pub event_name: String,
    #[sql_type = "dUuid"]
    pub organization_id: Uuid,
    #[sql_type = "BigInt"]
    pub allocation_count: i64,
    #[sql_type = "BigInt"]
    pub unpurchased_count: i64,
    #[sql_type = "BigInt"]
    pub available_count: i64,
    #[sql_type = "BigInt"]
    pub comp_count: i64,
    #[sql_type = "BigInt"]
    pub comp_available_count: i64,
    #[sql_type = "BigInt"]
    pub comp_redeemed_count: i64,
    #[sql_type = "BigInt"]
    pub comp_purchased_count: i64,
    #[sql_type = "BigInt"]
    pub comp_reserved_count: i64,
    #[sql_type = "BigInt"]
    pub comp_nullified_count: i64,
    #[sql_type = "BigInt"]
    pub hold_count: i64,
    #[sql_type = "BigInt"]
    pub hold_available_count: i64,
    #[sql_type = "BigInt"]
    pub hold_redeemed_count: i64,
    #[sql_type = "BigInt"]
    pub hold_purchased_count: i64,
    #[sql_type = "BigInt"]
    pub hold_reserved_count: i64,
    #[sql_type = "BigInt"]
    pub hold_nullified_count: i64,
}

#[derive(Serialize, Deserialize, PartialEq, Queryable, QueryableByName)]
pub struct TicketCountSalesRow {
    #[sql_type = "dUuid"]
    pub event_id: Uuid,
    #[sql_type = "dUuid"]
    pub organization_id: Uuid,
    #[sql_type = "dUuid"]
    pub ticket_type_id: Uuid,
    #[sql_type = "BigInt"]
    pub box_office_count: i64,
    #[sql_type = "BigInt"]
    pub online_count: i64,
    #[sql_type = "BigInt"]
    pub box_office_refunded_count: i64,
    #[sql_type = "BigInt"]
    pub online_refunded_count: i64,
    #[sql_type = "BigInt"]
    pub box_office_actual_count: i64,
    #[sql_type = "BigInt"]
    pub online_actual_count: i64,
    #[sql_type = "BigInt"]
    pub box_office_sales_in_cents: i64,
    #[sql_type = "BigInt"]
    pub online_sales_in_cents: i64,
    #[sql_type = "BigInt"]
    pub total_box_office_fees_in_cents: i64,
    #[sql_type = "BigInt"]
    pub total_online_fees_in_cents: i64,
    #[sql_type = "BigInt"]
    pub company_box_office_fees_in_cents: i64,
    #[sql_type = "BigInt"]
    pub client_box_office_fees_in_cents: i64,
    #[sql_type = "BigInt"]
    pub company_online_fees_in_cents: i64,
    #[sql_type = "BigInt"]
    pub client_online_fees_in_cents: i64,
}

impl Report {
    pub fn transaction_detail_report(
        event_id: Option<Uuid>,
        organization_id: Option<Uuid>,
        start: Option<NaiveDateTime>,
        end: Option<NaiveDateTime>,
        conn: &PgConnection,
    ) -> Result<Vec<TransactionReportRow>, DatabaseError> {
        let query = include_str!("../queries/reports/reports_transaction_details.sql");
        let q = diesel::sql_query(query)
            .bind::<Nullable<dUuid>, _>(event_id)
            .bind::<Nullable<dUuid>, _>(organization_id)
            .bind::<Nullable<Timestamp>, _>(start)
            .bind::<Nullable<Timestamp>, _>(end);
        let transaction_rows: Vec<TransactionReportRow> = q
            .get_results(conn)
            .to_db_error(ErrorCode::QueryError, "Could not fetch report results")?;
        Ok(transaction_rows)
    }

    pub fn summary_event_report(
        event_id: Uuid,
        start: Option<NaiveDateTime>,
        end: Option<NaiveDateTime>,
        conn: &PgConnection,
    ) -> Result<EventSummarySalesResult, DatabaseError> {
        let mut results =
            Report::summary_event_report_core(Some(event_id), None, start, end, conn)?;

        let result = match results.is_empty() {
            true => {
                let mut event_summary = EventSummarySalesResult {
                    ..Default::default()
                };
                event_summary.event_id = event_id;
                event_summary
            }
            false => results.pop().unwrap(),
        };

        Ok(result)
    }

    pub fn organization_summary_report(
        organization_id: Uuid,
        start: Option<NaiveDateTime>,
        end: Option<NaiveDateTime>,
        conn: &PgConnection,
    ) -> Result<Vec<EventSummarySalesResult>, DatabaseError> {
        Report::summary_event_report_core(None, Some(organization_id), start, end, conn)
    }

    fn summary_event_report_core(
        event_id: Option<Uuid>,
        organization_id: Option<Uuid>,
        start: Option<NaiveDateTime>,
        end: Option<NaiveDateTime>,
        conn: &PgConnection,
    ) -> Result<Vec<EventSummarySalesResult>, DatabaseError> {
        //First get the sales summary
        let query_sales = include_str!("../queries/reports/reports_event_summary_sales.sql");
        let q = diesel::sql_query(query_sales)
            .bind::<Nullable<dUuid>, _>(event_id)
            .bind::<Nullable<dUuid>, _>(organization_id)
            .bind::<Nullable<Timestamp>, _>(start)
            .bind::<Nullable<Timestamp>, _>(end);

        let sales_rows: Vec<EventSummarySalesRow> = q.get_results(conn).to_db_error(
            ErrorCode::QueryError,
            "Could not fetch report sales results",
        )?;

        //If there were no sales, return immediately
        if sales_rows.is_empty() {
            let empty_result: Vec<EventSummarySalesResult> = Vec::new();
            return Ok(empty_result);
        }

        let sales_rows = sales_rows.into_iter().group_by(|row| row.event_id);

        //Now get the transaction fees results
        let query_fees = include_str!("../queries/reports/reports_event_summary_fees.sql");
        let q = diesel::sql_query(query_fees)
            .bind::<Nullable<dUuid>, _>(event_id)
            .bind::<Nullable<dUuid>, _>(organization_id)
            .bind::<Nullable<Timestamp>, _>(start)
            .bind::<Nullable<Timestamp>, _>(end);

        let fees_rows: Vec<EventSummaryFeesRow> = q
            .get_results(conn)
            .to_db_error(ErrorCode::QueryError, "Could not fetch report fee results")?;

        let mut fees_hash: HashMap<Uuid, Vec<EventSummaryFeesRow>> = HashMap::new();
        for row in fees_rows {
            fees_hash
                .entry(row.event_id)
                .or_insert(Vec::<EventSummaryFeesRow>::new())
                .push(row);
        }

        //Now get the other fees results
        let query_other_fees =
            include_str!("../queries/reports/reports_event_summary_other_fees.sql");
        let q = diesel::sql_query(query_other_fees)
            .bind::<Nullable<dUuid>, _>(event_id)
            .bind::<Nullable<dUuid>, _>(organization_id)
            .bind::<Nullable<Timestamp>, _>(start)
            .bind::<Nullable<Timestamp>, _>(end);

        let other_fees_rows: Vec<EventSummaryOtherFees> = q
            .get_results(conn)
            .to_db_error(ErrorCode::QueryError, "Could not fetch report fee results")?;

        let mut other_fees_hash: HashMap<Uuid, Vec<EventSummaryOtherFees>> = HashMap::new();
        for row in other_fees_rows {
            other_fees_hash
                .entry(row.event_id)
                .or_insert(Vec::<EventSummaryOtherFees>::new())
                .push(row);
        }

        let mut result = Vec::<EventSummarySalesResult>::new();

        // assume that an event must have sales in order to have other fees
        for (event_id, sales) in sales_rows.into_iter() {
            result.push(EventSummarySalesResult {
                event_id,
                sales: sales.into_iter().collect_vec(),
                ticket_fees: fees_hash.get(&event_id).unwrap_or(&vec![]).to_vec(),
                other_fees: other_fees_hash.get(&event_id).unwrap_or(&vec![]).to_vec(),
            })
        }
        //Then get the fees summary
        Ok(result)
    }

    pub fn ticket_count_report(
        event_id: Option<Uuid>,
        organization_id: Option<Uuid>,
        conn: &PgConnection,
    ) -> Result<HashMap<Uuid, HashMap<Uuid, HashMap<Uuid, TicketCountResult>>>, DatabaseError> {
        let query_ticket_sales = include_str!("../queries/reports/reports-ticket-count-sales.sql");
        let q = diesel::sql_query(query_ticket_sales)
            .bind::<Nullable<dUuid>, _>(event_id)
            .bind::<Nullable<dUuid>, _>(organization_id);
        let count_sales_rows: Vec<TicketCountSalesRow> = q.get_results(conn).to_db_error(
            ErrorCode::QueryError,
            "Could not fetch ticket count results",
        )?;

        let mut result: HashMap<Uuid, HashMap<Uuid, HashMap<Uuid, TicketCountResult>>> =
            HashMap::new();

        for row in count_sales_rows {
            let mut org_key = result.entry(row.organization_id).or_insert(HashMap::new());
            let mut event_key = org_key.entry(row.event_id).or_insert(HashMap::new());
            event_key.insert(
                row.ticket_type_id,
                TicketCountResult {
                    sales: Some(row),
                    totals: None,
                },
            );
        }

        let query_ticket_counts = include_str!("../queries/reports/reports-ticket-count.sql");
        let q = diesel::sql_query(query_ticket_counts)
            .bind::<Nullable<dUuid>, _>(event_id)
            .bind::<Nullable<dUuid>, _>(organization_id);

        let count_rows: Vec<TicketCountRow> = q.get_results(conn).to_db_error(
            ErrorCode::QueryError,
            "Could not fetch ticket count results",
        )?;

        for row in count_rows {
            let mut org_key = result.entry(row.organization_id).or_insert(HashMap::new());
            let mut event_key = org_key.entry(row.event_id).or_insert(HashMap::new());
            let mut ticket_key = event_key
                .entry(row.ticket_type_id)
                .or_insert(TicketCountResult {
                    sales: None,
                    totals: None,
                });
            ticket_key.totals = Some(row);
        }

        Ok(result)
    }
}
