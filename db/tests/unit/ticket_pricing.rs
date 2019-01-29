use bigneon_db::dev::TestProject;
use bigneon_db::models::*;
use bigneon_db::utils::errors::ErrorCode::ValidationError;
use chrono::NaiveDate;

#[test]
fn create() {
    let project = TestProject::new();
    let event = project.create_event().with_tickets().finish();
    let ticket_type = &event
        .ticket_types(true, None, project.get_connection())
        .unwrap()[0];
    let sd1 = NaiveDate::from_ymd(2016, 7, 8).and_hms(4, 10, 11);
    let ed1 = NaiveDate::from_ymd(2016, 7, 9).and_hms(4, 10, 11);
    let sd2 = NaiveDate::from_ymd(2016, 7, 9).and_hms(4, 10, 11);
    let ed2 = NaiveDate::from_ymd(2016, 7, 10).and_hms(4, 10, 11);

    let ticket_pricing = TicketPricing::create(
        ticket_type.id,
        "Early Bird".to_string(),
        sd1,
        ed1,
        100,
        false,
        None,
    )
    .commit(project.get_connection())
    .unwrap();

    let pricing2 = TicketPricing::create(
        ticket_type.id,
        "Wormless Bird".to_string(),
        sd2,
        ed2,
        500,
        false,
        None,
    )
    .commit(project.get_connection())
    .unwrap();

    let pricing = ticket_type
        .ticket_pricing(project.get_connection())
        .unwrap();
    assert_eq!(pricing, vec![ticket_pricing, pricing2]);
}

#[test]
fn ticket_pricing_no_overlapping_periods() {
    let project = TestProject::new();
    let event = project.create_event().with_tickets().finish();
    let ticket_type = &event
        .ticket_types(true, None, project.get_connection())
        .unwrap()[0];
    let start_date1 = NaiveDate::from_ymd(2016, 7, 6).and_hms(4, 10, 11);
    let end_date1 = NaiveDate::from_ymd(2016, 7, 10).and_hms(4, 10, 11);
    let start_date2 = NaiveDate::from_ymd(2016, 7, 7).and_hms(4, 10, 11);
    let end_date2 = NaiveDate::from_ymd(2016, 7, 8).and_hms(4, 10, 11);
    let start_date3 = NaiveDate::from_ymd(2016, 8, 7).and_hms(4, 10, 11);
    let end_date3 = NaiveDate::from_ymd(2016, 8, 9).and_hms(4, 10, 11);
    let ticket_pricing1 = TicketPricing::create(
        ticket_type.id,
        "Early Bird".to_string(),
        start_date1,
        end_date1,
        100,
        false,
        None,
    )
    .commit(project.get_connection())
    .unwrap();

    let ticket_pricing2 = TicketPricing::create(
        ticket_type.id,
        "Early Bird".to_string(),
        start_date2,
        end_date2,
        100,
        false,
        None,
    )
    .commit(project.get_connection())
    .unwrap();

    let ticket_pricing3 = TicketPricing::create(
        ticket_type.id,
        "Early Bird".to_string(),
        start_date3,
        end_date3,
        100,
        false,
        None,
    )
    .commit(project.get_connection())
    .unwrap();

    // ticket_pricing1 and ticket_pricing2 overlap
    assert!(TicketPricing::ticket_pricing_no_overlapping_periods(
        ticket_pricing1.id,
        ticket_type.id,
        start_date1,
        end_date1,
        false,
        TicketPricingStatus::Published,
        project.get_connection()
    )
    .unwrap()
    .is_err());
    assert!(TicketPricing::ticket_pricing_no_overlapping_periods(
        ticket_pricing2.id,
        ticket_type.id,
        start_date2,
        end_date2,
        false,
        TicketPricingStatus::Published,
        project.get_connection()
    )
    .unwrap()
    .is_err());

    // ticket_pricing3 does not overlap
    assert!(TicketPricing::ticket_pricing_no_overlapping_periods(
        ticket_pricing3.id,
        ticket_type.id,
        start_date3,
        end_date3,
        false,
        TicketPricingStatus::Published,
        project.get_connection()
    )
    .unwrap()
    .is_ok());
}

#[test]
fn create_with_same_date_validation_errors() {
    let project = TestProject::new();
    let event = project.create_event().with_tickets().finish();
    let ticket_type = &event
        .ticket_types(true, None, project.get_connection())
        .unwrap()[0];
    let same_date = NaiveDate::from_ymd(2016, 7, 9).and_hms(4, 10, 11);

    let mut ticket_pricing = TicketPricing::create(
        ticket_type.id,
        "Early Bird".to_string(),
        same_date,
        same_date,
        100,
        false,
        None,
    );

    let result = ticket_pricing.clone().commit(project.get_connection());
    match result {
        Ok(_) => {
            panic!("Expected validation error");
        }
        Err(error) => match &error.error_code {
            ValidationError { errors } => {
                assert!(errors.contains_key("ticket_pricing.start_date"));
                assert_eq!(errors["ticket_pricing.start_date"].len(), 1);
                assert_eq!(
                    errors["ticket_pricing.start_date"][0].code,
                    "start_date_must_be_before_end_date"
                );
                assert_eq!(
                    &errors["ticket_pricing.start_date"][0]
                        .message
                        .clone()
                        .unwrap()
                        .into_owned(),
                    "Start date must be before end date"
                );
            }
            _ => panic!("Expected validation error"),
        },
    }

    // Period without start date validation
    ticket_pricing.start_date = same_date;
    ticket_pricing.end_date = NaiveDate::from_ymd(2016, 7, 15).and_hms(4, 10, 11);
    let result = ticket_pricing.clone().commit(project.get_connection());
    assert!(result.is_ok());
}

#[test]
fn create_with_validation_errors() {
    let project = TestProject::new();
    let event = project.create_event().with_tickets().finish();
    let ticket_type = &event
        .ticket_types(true, None, project.get_connection())
        .unwrap()[0];
    let start_date1 = NaiveDate::from_ymd(2016, 7, 6).and_hms(4, 10, 11);
    let end_date1 = NaiveDate::from_ymd(2016, 7, 10).and_hms(4, 10, 11);
    let start_date2 = NaiveDate::from_ymd(2016, 7, 9).and_hms(4, 10, 11);
    let end_date2 = NaiveDate::from_ymd(2016, 7, 8).and_hms(4, 10, 11);
    TicketPricing::create(
        ticket_type.id,
        "Early Bird".to_string(),
        start_date1,
        end_date1,
        100,
        false,
        None,
    )
    .commit(project.get_connection())
    .unwrap();

    let mut ticket_pricing = TicketPricing::create(
        ticket_type.id,
        "Early Bird".to_string(),
        start_date2,
        end_date2,
        100,
        false,
        None,
    );

    let result = ticket_pricing.clone().commit(project.get_connection());
    match result {
        Ok(_) => {
            panic!("Expected validation error");
        }
        Err(error) => match &error.error_code {
            ValidationError { errors } => {
                assert!(errors.contains_key("ticket_pricing.start_date"));
                assert_eq!(errors["ticket_pricing.start_date"].len(), 1);
                assert_eq!(
                    errors["ticket_pricing.start_date"][0].code,
                    "start_date_must_be_before_end_date"
                );
                assert_eq!(
                    &errors["ticket_pricing.start_date"][0]
                        .message
                        .clone()
                        .unwrap()
                        .into_owned(),
                    "Start date must be before end date"
                );
            }
            _ => panic!("Expected validation error"),
        },
    }

    // Period without start date validation
    ticket_pricing.start_date = end_date1;
    ticket_pricing.end_date = NaiveDate::from_ymd(2016, 7, 15).and_hms(4, 10, 11);
    let result = ticket_pricing.clone().commit(project.get_connection());
    assert!(result.is_ok());
}

#[test]
fn update_with_validation_errors() {
    let project = TestProject::new();
    let event = project.create_event().with_tickets().finish();
    let ticket_type = &event
        .ticket_types(true, None, project.get_connection())
        .unwrap()[0];
    let start_date1 = NaiveDate::from_ymd(2016, 7, 6).and_hms(4, 10, 11);
    let end_date1 = NaiveDate::from_ymd(2016, 7, 10).and_hms(4, 10, 11);
    let start_date2 = NaiveDate::from_ymd(2016, 7, 10).and_hms(4, 10, 11);
    let end_date2 = NaiveDate::from_ymd(2016, 7, 11).and_hms(4, 10, 11);
    TicketPricing::create(
        ticket_type.id,
        "Early Bird".to_string(),
        start_date1,
        end_date1,
        100,
        false,
        None,
    )
    .commit(project.get_connection())
    .unwrap();
    let ticket_pricing = TicketPricing::create(
        ticket_type.id,
        "Regular".to_string(),
        start_date2,
        end_date2,
        100,
        false,
        None,
    )
    .commit(project.get_connection())
    .unwrap();

    let mut ticket_pricing_parameters: TicketPricingEditableAttributes = Default::default();
    ticket_pricing_parameters.start_date = Some(NaiveDate::from_ymd(2016, 7, 9).and_hms(4, 10, 11));
    ticket_pricing_parameters.end_date = Some(NaiveDate::from_ymd(2016, 7, 8).and_hms(4, 10, 11));
    let result = ticket_pricing.update(ticket_pricing_parameters.clone(), project.get_connection());
    match result {
        Ok(_) => {
            panic!("Expected validation error");
        }
        Err(error) => match &error.error_code {
            ValidationError { errors } => {
                assert!(errors.contains_key("ticket_pricing.start_date"));
                assert_eq!(errors["ticket_pricing.start_date"].len(), 1);
                assert_eq!(
                    errors["ticket_pricing.start_date"][0].code,
                    "start_date_must_be_before_end_date"
                );
                assert_eq!(
                    &errors["ticket_pricing.start_date"][0]
                        .message
                        .clone()
                        .unwrap()
                        .into_owned(),
                    "Start date must be before end date"
                );
            }
            _ => panic!("Expected validation error"),
        },
    }

    // Updates without start date validation triggering
    ticket_pricing_parameters.start_date = Some(end_date1);
    ticket_pricing_parameters.end_date = Some(NaiveDate::from_ymd(2016, 7, 15).and_hms(4, 10, 11));
    let result = ticket_pricing.update(ticket_pricing_parameters.clone(), project.get_connection());
    assert!(result.is_ok());
}

#[test]
fn update() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let event = project.create_event().with_tickets().finish();
    let ticket_type = &event.ticket_types(true, None, connection).unwrap()[0];
    let start_date = NaiveDate::from_ymd(2016, 7, 8).and_hms(4, 10, 11);
    let end_date = NaiveDate::from_ymd(2016, 7, 9).and_hms(4, 10, 11);
    let ticket_pricing = TicketPricing::create(
        ticket_type.id,
        "Early Bird".to_string(),
        start_date,
        end_date,
        100,
        false,
        None,
    )
    .commit(connection)
    .unwrap();
    //Change editable parameters and submit ticket pricing update request
    let update_name = String::from("updated_event_name");
    let update_price_in_cents: i64 = 200;
    let update_start_date = NaiveDate::from_ymd(2018, 4, 23).and_hms(5, 14, 18);
    let update_end_date = NaiveDate::from_ymd(2018, 6, 1).and_hms(8, 5, 34);
    let update_parameters = TicketPricingEditableAttributes {
        name: Some(update_name.clone()),
        price_in_cents: Some(update_price_in_cents),
        start_date: Some(update_start_date),
        end_date: Some(update_end_date),
        is_box_office_only: Some(false),
    };
    let updated_ticket_pricing = ticket_pricing
        .update(update_parameters, connection)
        .unwrap();
    assert_eq!(updated_ticket_pricing.id, ticket_pricing.id);
    assert_eq!(updated_ticket_pricing.name, update_name);
    assert_eq!(updated_ticket_pricing.price_in_cents, update_price_in_cents);
    assert_eq!(updated_ticket_pricing.start_date, update_start_date);
    assert_eq!(updated_ticket_pricing.end_date, update_end_date);
    assert_eq!(updated_ticket_pricing.is_box_office_only, false);
    assert_eq!(
        updated_ticket_pricing.ticket_type_id,
        ticket_pricing.ticket_type_id
    );
}

#[test]
fn update_with_affected_orders() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let event = project.create_event().with_tickets().finish();
    let ticket_type = &event.ticket_types(true, None, connection).unwrap()[0];
    let start_date = NaiveDate::from_ymd(2016, 7, 8).and_hms(4, 10, 11);
    let end_date = NaiveDate::from_ymd(2088, 7, 9).and_hms(4, 10, 11);
    let ticket_pricing = TicketPricing::create(
        ticket_type.id,
        "Early Bird".to_string(),
        start_date,
        end_date,
        100,
        false,
        None,
    )
    .commit(connection)
    .unwrap();

    let user = project.create_user().finish();
    let mut cart = Order::find_or_create_cart(&user, connection).unwrap();
    cart.update_quantities(
        &[UpdateOrderItem {
            ticket_type_id: ticket_type.id,
            quantity: 10,
            redemption_code: None,
        }],
        false,
        false,
        connection,
    )
    .unwrap();
    let items = cart.items(&connection).unwrap();
    let order_item = items
        .iter()
        .find(|i| i.ticket_type_id == Some(ticket_type.id))
        .unwrap();
    assert_eq!(order_item.ticket_pricing_id, Some(ticket_pricing.id));

    //Change editable parameters and submit ticket pricing update request
    let update_name = String::from("updated_event_name");
    let update_price_in_cents: i64 = 200;
    let update_start_date = NaiveDate::from_ymd(2018, 4, 23).and_hms(5, 14, 18);
    let update_end_date = NaiveDate::from_ymd(2018, 6, 1).and_hms(8, 5, 34);
    let update_parameters = TicketPricingEditableAttributes {
        name: Some(update_name.clone()),
        price_in_cents: Some(update_price_in_cents),
        start_date: Some(update_start_date),
        end_date: Some(update_end_date),
        is_box_office_only: Some(false),
    };
    let updated_ticket_pricing = ticket_pricing
        .update(update_parameters, connection)
        .unwrap();

    // ID should be new but everything else should match updated logic
    assert_ne!(updated_ticket_pricing.id, ticket_pricing.id);
    assert_eq!(updated_ticket_pricing.name, update_name);
    assert_eq!(updated_ticket_pricing.price_in_cents, update_price_in_cents);
    assert_eq!(updated_ticket_pricing.start_date, update_start_date);
    assert_eq!(updated_ticket_pricing.end_date, update_end_date);
    assert_eq!(updated_ticket_pricing.is_box_office_only, false);
    assert_eq!(
        updated_ticket_pricing.ticket_type_id,
        ticket_pricing.ticket_type_id
    );

    // Reloading existing should show nothing has changed but status is now deleted
    let old_ticket_pricing = TicketPricing::find(ticket_pricing.id, connection).unwrap();
    assert_eq!(old_ticket_pricing.id, ticket_pricing.id);
    assert_eq!(old_ticket_pricing.name, ticket_pricing.name);
    assert_eq!(
        old_ticket_pricing.price_in_cents,
        ticket_pricing.price_in_cents
    );
    assert_eq!(old_ticket_pricing.start_date, ticket_pricing.start_date);
    assert_eq!(old_ticket_pricing.end_date, ticket_pricing.end_date);
    assert_eq!(
        old_ticket_pricing.is_box_office_only,
        ticket_pricing.is_box_office_only
    );
    assert_eq!(
        old_ticket_pricing.ticket_type_id,
        ticket_pricing.ticket_type_id
    );
    assert_eq!(old_ticket_pricing.status, TicketPricingStatus::Deleted);
}

#[test]
fn remove() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let event = project.create_event().with_tickets().finish();
    let ticket_type = &event.ticket_types(true, None, connection).unwrap()[0];
    let start_date = NaiveDate::from_ymd(2016, 7, 8).and_hms(4, 10, 11);
    let end_date = NaiveDate::from_ymd(2016, 7, 9).and_hms(4, 10, 11);
    let ticket_pricing1 = TicketPricing::create(
        ticket_type.id,
        "Early Bird".to_string(),
        start_date,
        end_date,
        100,
        false,
        None,
    )
    .commit(connection)
    .unwrap();

    let start_date = NaiveDate::from_ymd(2016, 7, 9).and_hms(4, 10, 11);
    let end_date = NaiveDate::from_ymd(2016, 7, 10).and_hms(4, 10, 11);
    let ticket_pricing2 = TicketPricing::create(
        ticket_type.id,
        "Standard".to_string(),
        start_date,
        end_date,
        200,
        false,
        None,
    )
    .commit(connection)
    .unwrap();
    //Remove ticket pricing and check if it is still available
    ticket_pricing1.destroy(connection).unwrap();
    let ticket_pricings = ticket_type.ticket_pricing(connection).unwrap();
    let found_index1 = ticket_pricings
        .iter()
        .position(|ref r| r.id == ticket_pricing1.id);
    let found_index2 = ticket_pricings
        .iter()
        .position(|ref r| r.id == ticket_pricing2.id);
    assert!(found_index1.is_none());
    assert!(found_index2.is_some());
}

#[test]
fn find() {
    let project = TestProject::new();
    let event = project.create_event().with_tickets().finish();
    let ticket_type = &event
        .ticket_types(true, None, project.get_connection())
        .unwrap()[0];
    let sd1 = NaiveDate::from_ymd(2016, 7, 8).and_hms(4, 10, 11);
    let ed1 = NaiveDate::from_ymd(2016, 7, 9).and_hms(4, 10, 11);
    let ticket_pricing = TicketPricing::create(
        ticket_type.id,
        "Early Bird".to_string(),
        sd1,
        ed1,
        100,
        false,
        None,
    )
    .commit(project.get_connection())
    .unwrap();
    let found_ticket_pricing =
        TicketPricing::find(ticket_pricing.id, project.get_connection()).unwrap();

    assert_eq!(found_ticket_pricing, ticket_pricing);
}

#[test]
fn get_current_ticket_pricing() {
    let project = TestProject::new();
    let admin = project.create_user().finish();

    let organization = project
        .create_organization()
        .with_fee_schedule(&project.create_fee_schedule().finish(admin.id))
        .finish();
    let event = project
        .create_event()
        .with_organization(&organization)
        .with_ticket_pricing()
        .finish();

    let ticket_types =
        TicketType::find_by_event_id(event.id, true, None, project.get_connection()).unwrap();

    let ticket_pricing = TicketPricing::get_current_ticket_pricing(
        ticket_types[0].id,
        false,
        false,
        project.get_connection(),
    )
    .unwrap();

    assert_eq!(ticket_pricing.name, "Standard".to_string())
}

#[test]
fn get_current_ticket_capacity() {
    let project = TestProject::new();

    let admin = project.create_user().finish();

    let organization = project
        .create_organization()
        .with_fee_schedule(&project.create_fee_schedule().finish(admin.id))
        .finish();
    let event = project
        .create_event()
        .with_organization(&organization)
        .with_ticket_pricing()
        .finish();
    let ticket_types =
        TicketType::find_by_event_id(event.id, true, None, project.get_connection()).unwrap();
    assert_eq!(ticket_types.len(), 1);

    let ticket_capacity = ticket_types[0]
        .ticket_capacity(project.get_connection())
        .unwrap();
    assert_eq!(ticket_capacity, 100);
}
