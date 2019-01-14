use bigneon_db::prelude::*;
use chrono::NaiveDateTime;
use diesel::PgConnection;
use models::DisplayTicketPricing;
use std::cmp;
use uuid::Uuid;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct UserDisplayTicketType {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: TicketTypeStatus,
    pub available: u32,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub increment: i32,
    pub limit_per_person: u32,
    pub ticket_pricing: Option<DisplayTicketPricing>,
    pub redemption_code: Option<String>,
}

impl UserDisplayTicketType {
    pub fn from_ticket_type(
        ticket_type: &TicketType,
        fee_schedule: &FeeSchedule,
        box_office_pricing: bool,
        conn: &PgConnection,
    ) -> Result<UserDisplayTicketType, DatabaseError> {
        UserDisplayTicketType::from_ticket_type_and_hold(
            ticket_type,
            fee_schedule,
            box_office_pricing,
            None,
            conn,
        )
    }

    pub fn from_ticket_type_and_hold(
        ticket_type: &TicketType,
        fee_schedule: &FeeSchedule,
        box_office_pricing: bool,
        hold: Option<Hold>,
        conn: &PgConnection,
    ) -> Result<UserDisplayTicketType, DatabaseError> {
        let mut status = ticket_type.status;
        let available = ticket_type.remaining_ticket_count(conn)?;

        let ticket_pricing = match ticket_type
            .current_ticket_pricing(box_office_pricing, conn)
            .optional()?
        {
            Some(ticket_pricing) => Some(DisplayTicketPricing::from_ticket_pricing(
                &ticket_pricing,
                fee_schedule,
                conn,
            )?),
            None => None,
        };

        if ticket_type.status == TicketTypeStatus::Published {
            if available == 0 {
                status = TicketTypeStatus::SoldOut;
            } else if ticket_pricing.is_none() {
                status = TicketTypeStatus::NoActivePricing;
            }
        }

        let mut result = UserDisplayTicketType {
            id: ticket_type.id,
            name: ticket_type.name.clone(),
            description: ticket_type.description.clone(),
            status,
            start_date: ticket_type.start_date,
            end_date: ticket_type.end_date,
            ticket_pricing,
            available,
            redemption_code: hold.as_ref().map(|h| h.redemption_code.to_string()),
            increment: ticket_type.increment,
            limit_per_person: ticket_type.limit_per_person as u32,
        };

        if let Some(h) = hold {
            result.description = Some(format!("Using promo code: {}", h.redemption_code));
            result.limit_per_person = h.max_per_order.unwrap_or(0) as u32;
            result.available = h.quantity(conn)?.1;
            match h.hold_type {
                HoldTypes::Comp => {
                    result.ticket_pricing = result.ticket_pricing.map(|tp| DisplayTicketPricing {
                        fee_in_cents: 0,
                        price_in_cents: 0,
                        ..tp
                    })
                }
                HoldTypes::Discount => {
                    result.ticket_pricing = result.ticket_pricing.map(|tp| DisplayTicketPricing {
                        price_in_cents: cmp::max(
                            0,
                            tp.price_in_cents - h.discount_in_cents.unwrap_or(0),
                        ),
                        ..tp
                    })
                }
            }
        }
        Ok(result)
    }
}
