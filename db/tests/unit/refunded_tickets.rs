use bigneon_db::dev::TestProject;
use bigneon_db::prelude::*;

#[test]
fn find_by_ticket_instance_ids() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let creator = project.create_user().finish();
    let organization = project
        .create_organization()
        .with_fee_schedule(&project.create_fee_schedule().finish(creator.id))
        .finish();
    let event = project
        .create_event()
        .with_organization(&organization)
        .with_ticket_pricing()
        .finish();
    let user = project.create_user().finish();
    project
        .create_order()
        .for_event(&event)
        .for_user(&user)
        .quantity(1)
        .is_paid()
        .finish();
    let tickets = TicketInstance::find_for_user(user.id, connection).unwrap();
    let ticket = &tickets[0];
    let refunded_ticket =
        RefundedTicket::find_or_create_by_ticket_instance(&ticket, connection).unwrap();
    let found_tickets =
        RefundedTicket::find_by_ticket_instance_ids(vec![ticket.id], connection).unwrap();
    assert_eq!(found_tickets, vec![refunded_ticket]);
}

#[test]
fn find_or_create_by_ticket_instance() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let creator = project.create_user().finish();
    let organization = project
        .create_organization()
        .with_fee_schedule(&project.create_fee_schedule().finish(creator.id))
        .finish();
    let event = project
        .create_event()
        .with_organization(&organization)
        .with_ticket_pricing()
        .finish();
    let user = project.create_user().finish();
    project
        .create_order()
        .for_event(&event)
        .for_user(&user)
        .quantity(1)
        .is_paid()
        .finish();
    let tickets = TicketInstance::find_for_user(user.id, connection).unwrap();
    let ticket = &tickets[0];
    let mut refunded_ticket =
        RefundedTicket::find_or_create_by_ticket_instance(&ticket, connection).unwrap();
    assert_eq!(ticket.order_item_id, Some(refunded_ticket.order_item_id));
    assert_eq!(ticket.id, refunded_ticket.ticket_instance_id);
    assert!(refunded_ticket.ticket_refunded_at.is_none());
    assert!(refunded_ticket.fee_refunded_at.is_none());

    refunded_ticket.mark_refunded(false, connection).unwrap();
    let refunded_ticket =
        RefundedTicket::find_or_create_by_ticket_instance(&ticket, connection).unwrap();
    assert_eq!(ticket.order_item_id, Some(refunded_ticket.order_item_id));
    assert_eq!(ticket.id, refunded_ticket.ticket_instance_id);
    assert!(refunded_ticket.ticket_refunded_at.is_some());
    assert!(refunded_ticket.fee_refunded_at.is_some());
}

#[test]
fn mark_refunded() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let creator = project.create_user().finish();
    let organization = project
        .create_organization()
        .with_fee_schedule(&project.create_fee_schedule().finish(creator.id))
        .finish();
    let event = project
        .create_event()
        .with_organization(&organization)
        .with_ticket_pricing()
        .finish();
    let user = project.create_user().finish();
    project
        .create_order()
        .for_event(&event)
        .for_user(&user)
        .quantity(2)
        .is_paid()
        .finish();
    let tickets = TicketInstance::find_for_user(user.id, connection).unwrap();
    let ticket = &tickets[0];
    let mut refunded_ticket = RefundedTicket::create(ticket.order_item_id.unwrap(), ticket.id)
        .commit(connection)
        .unwrap();

    assert_eq!(ticket.order_item_id, Some(refunded_ticket.order_item_id));
    assert_eq!(ticket.id, refunded_ticket.ticket_instance_id);
    assert!(refunded_ticket.ticket_refunded_at.is_none());
    assert!(refunded_ticket.fee_refunded_at.is_none());

    // Refunding fee and subsequently refunding just the ticket fee
    refunded_ticket.mark_refunded(true, connection).unwrap();
    assert!(refunded_ticket.ticket_refunded_at.is_none());
    assert!(refunded_ticket.fee_refunded_at.is_some());

    // Refunding ticket as well
    refunded_ticket.mark_refunded(false, connection).unwrap();
    assert!(refunded_ticket.ticket_refunded_at.is_some());
    assert!(refunded_ticket.fee_refunded_at.is_some());

    // Refunding both ticket and fee at once
    let ticket2 = &tickets[1];
    let mut refunded_ticket = RefundedTicket::create(ticket2.order_item_id.unwrap(), ticket2.id)
        .commit(connection)
        .unwrap();
    assert_eq!(ticket.order_item_id, Some(refunded_ticket.order_item_id));
    assert_eq!(ticket2.id, refunded_ticket.ticket_instance_id);
    assert!(refunded_ticket.ticket_refunded_at.is_none());
    assert!(refunded_ticket.fee_refunded_at.is_none());
    refunded_ticket.mark_refunded(false, connection).unwrap();
    assert!(refunded_ticket.ticket_refunded_at.is_some());
    assert!(refunded_ticket.fee_refunded_at.is_some());
}
