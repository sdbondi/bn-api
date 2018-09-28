use actix_web::Query;
use actix_web::{HttpResponse, Json, Path};
use auth::user::User;
use bigneon_db::models::*;
use chrono::prelude::*;
use db::Connection;
use errors::*;
use helpers::application;
use models::{
    AdminDisplayTicketType, EventTicketPathParameters, PathParameters, UserDisplayTicketType,
};
use serde_with::{self, CommaSeparator};
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize)]
pub struct SearchParameters {
    query: Option<String>,
    region_id: Option<Uuid>,
    #[serde(
        default,
        with = "serde_with::rust::StringWithSeparator::<CommaSeparator>"
    )]
    status: Vec<EventStatus>,
    start_utc: Option<NaiveDateTime>,
    end_utc: Option<NaiveDateTime>,
}

#[derive(Deserialize)]
pub struct PagingSearchParameters {
    pub from_index: usize,
    pub to_index: usize,
}

#[derive(Deserialize, Debug)]
pub struct AddArtistRequest {
    pub artist_id: Uuid,
    pub rank: i32,
    pub set_time: Option<NaiveDateTime>,
}

#[derive(Deserialize, Debug)]
pub struct UpdateArtistsRequest {
    pub artist_id: Uuid,
    pub set_time: Option<NaiveDateTime>,
}

pub fn index(
    (connection, parameters, user): (Connection, Query<SearchParameters>, Option<User>),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let parameters = parameters.into_inner();
    let events = Event::search(
        parameters.query,
        parameters.region_id,
        parameters.start_utc,
        parameters.end_utc,
        if parameters.status.is_empty() {
            None
        } else {
            Some(parameters.status)
        },
        user.map(|u| u.user),
        connection,
    )?;

    #[derive(Serialize)]
    struct EventVenueEntry {
        id: Uuid,
        name: String,
        organization_id: Uuid,
        venue_id: Option<Uuid>,
        created_at: NaiveDateTime,
        event_start: Option<NaiveDateTime>,
        door_time: Option<NaiveDateTime>,
        status: String,
        publish_date: Option<NaiveDateTime>,
        promo_image_url: Option<String>,
        additional_info: Option<String>,
        age_limit: Option<i32>,
        cancelled_at: Option<NaiveDateTime>,
        venue: Option<Venue>,
    }

    let mut results: Vec<EventVenueEntry> = Vec::new();
    for e in events {
        results.push(EventVenueEntry {
            venue: match e.venue_id {
                Some(v) => Some(Venue::find(v, connection)?),
                None => None,
            },
            id: e.id,
            name: e.name,
            organization_id: e.organization_id,
            venue_id: e.venue_id,
            created_at: e.created_at,
            event_start: e.event_start,
            door_time: e.door_time,
            status: e.status,
            publish_date: e.publish_date,
            promo_image_url: e.promo_image_url,
            additional_info: e.additional_info,
            age_limit: e.age_limit,
            cancelled_at: e.cancelled_at,
        })
    }

    Ok(HttpResponse::Ok().json(&results))
}

pub fn show(
    (connection, parameters, user): (Connection, Path<PathParameters>, Option<User>),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let event = Event::find(parameters.id, connection)?;
    let organization = event.organization(connection)?;

    let venue = event.venue(connection)?;
    let event_artists = EventArtist::find_all_from_event(event.id, connection)?;
    let total_interest = EventInterest::total_interest(event.id, connection)?;
    let user_interest = match user {
        Some(u) => EventInterest::user_interest(event.id, u.id(), connection)?,
        None => false,
    };

    let ticket_types = TicketType::find_by_event_id(parameters.id, connection)?;
    let mut display_ticket_types = Vec::new();
    for ticket_type in ticket_types {
        display_ticket_types.push(UserDisplayTicketType::from_ticket_type(
            &ticket_type,
            connection,
        )?);
    }

    //This struct is used to just contain the id and name of the org
    #[derive(Serialize)]
    struct ShortOrganization {
        id: Uuid,
        name: String,
    }
    #[derive(Serialize)]
    struct DisplayEventArtist {
        event_id: Uuid,
        artist_id: Uuid,
        rank: i32,
        set_time: Option<NaiveDateTime>,
    }
    #[derive(Serialize)]
    struct R {
        id: Uuid,
        name: String,
        organization_id: Uuid,
        venue_id: Option<Uuid>,
        created_at: NaiveDateTime,
        event_start: Option<NaiveDateTime>,
        door_time: Option<NaiveDateTime>,
        status: String,
        publish_date: Option<NaiveDateTime>,
        promo_image_url: Option<String>,
        additional_info: Option<String>,
        age_limit: Option<i32>,
        organization: ShortOrganization,
        venue: Option<Venue>,
        artists: Vec<DisplayEventArtist>,
        ticket_types: Vec<UserDisplayTicketType>,
        total_interest: u32,
        user_is_interested: bool,
    }

    let display_event_artists = event_artists
        .iter()
        .map(|e| DisplayEventArtist {
            event_id: e.event_id,
            artist_id: e.artist_id,
            rank: e.rank,
            set_time: e.set_time,
        }).collect();

    Ok(HttpResponse::Ok().json(&R {
        id: event.id,
        name: event.name,
        organization_id: event.organization_id,
        venue_id: event.venue_id,
        created_at: event.created_at,
        event_start: event.event_start,
        door_time: event.door_time,
        status: event.status,
        publish_date: event.publish_date,
        promo_image_url: event.promo_image_url,
        additional_info: event.additional_info,
        age_limit: event.age_limit,
        organization: ShortOrganization {
            id: organization.id,
            name: organization.name,
        },
        venue: venue,
        artists: display_event_artists,
        ticket_types: display_ticket_types,
        total_interest: total_interest,
        user_is_interested: user_interest,
    }))
}

pub fn show_from_organizations(
    (connection, organization_id): (Connection, Path<PathParameters>),
) -> Result<HttpResponse, BigNeonError> {
    let events = Event::find_all_events_from_organization(&organization_id.id, connection.get())?;
    Ok(HttpResponse::Ok().json(&events))
}

pub fn show_from_venues(
    (connection, venue_id): (Connection, Path<PathParameters>),
) -> Result<HttpResponse, BigNeonError> {
    let events = Event::find_all_events_from_venue(&venue_id.id, connection.get())?;
    Ok(HttpResponse::Ok().json(&events))
}

pub fn create(
    (connection, new_event, user): (Connection, Json<NewEvent>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    if !user.has_scope(Scopes::EventWrite, None, connection)? {
        if !user.has_scope(
            Scopes::EventWrite,
            Some(&Organization::find(new_event.organization_id, connection)?),
            connection,
        )? {
            return application::unauthorized();
        }
    }

    match new_event.validate() {
        Ok(_) => {
            let event = new_event.commit(connection)?;
            Ok(HttpResponse::Created().json(&event))
        }
        Err(e) => application::validation_error_response(e),
    }
}

pub fn update(
    (connection, parameters, event_parameters, user): (
        Connection,
        Path<PathParameters>,
        Json<EventEditableAttributes>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let event = Event::find(parameters.id, connection)?;
    if !user.has_scope(
        Scopes::EventWrite,
        Some(&event.organization(connection)?),
        connection,
    )? {
        return application::unauthorized();
    }

    match event_parameters.validate() {
        Ok(_) => {
            let updated_event = event.update(event_parameters.into_inner(), connection)?;
            Ok(HttpResponse::Ok().json(&updated_event))
        }
        Err(e) => application::validation_error_response(e),
    }
}

pub fn cancel(
    (connection, parameters, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let event = Event::find(parameters.id, connection)?;
    if !user.has_scope(
        Scopes::EventWrite,
        Some(&event.organization(connection)?),
        connection,
    )? {
        return application::unauthorized();
    }

    //Doing this in the DB layer so it can use the DB time as now.
    let updated_event = event.cancel(connection)?;

    Ok(HttpResponse::Ok().json(&updated_event))
}

pub fn list_interested_users(
    (connection, path_parameters, query_parameters, user): (
        Connection,
        Path<PathParameters>,
        Query<PagingSearchParameters>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    if !user.has_scope(Scopes::EventInterest, None, connection)? {
        return application::unauthorized();
    }

    let query_parameters = query_parameters.into_inner();
    let event_interested_users = EventInterest::list_interested_users(
        path_parameters.id,
        user.id(),
        query_parameters.from_index,
        query_parameters.to_index,
        connection,
    )?;
    Ok(HttpResponse::Ok().json(&event_interested_users))
}

pub fn add_interest(
    (connection, parameters, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    if !user.has_scope(Scopes::EventInterest, None, connection)? {
        return application::unauthorized();
    }

    let event_interest = EventInterest::create(parameters.id, user.id()).commit(connection)?;
    Ok(HttpResponse::Created().json(&event_interest))
}

pub fn remove_interest(
    (connection, parameters, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    if !user.has_scope(Scopes::EventInterest, None, connection)? {
        return application::unauthorized();
    }

    let event_interest = EventInterest::remove(parameters.id, user.id(), connection)?;
    Ok(HttpResponse::Ok().json(&event_interest))
}

pub fn add_artist(
    (connection, parameters, event_artist, user): (
        Connection,
        Path<PathParameters>,
        Json<AddArtistRequest>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let event = Event::find(parameters.id, connection)?;
    if !user.has_scope(
        Scopes::EventWrite,
        Some(&event.organization(connection)?),
        connection,
    )? {
        return application::unauthorized();
    }

    let event_artist = EventArtist::create(
        parameters.id,
        event_artist.artist_id,
        event_artist.rank,
        event_artist.set_time,
    ).commit(connection)?;
    Ok(HttpResponse::Created().json(&event_artist))
}

pub fn update_artists(
    (connection, parameters, artists, user): (
        Connection,
        Path<PathParameters>,
        Json<Vec<UpdateArtistsRequest>>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let event = Event::find(parameters.id, connection)?;
    if !user.has_scope(
        Scopes::EventWrite,
        Some(&event.organization(connection)?),
        connection,
    )? {
        return application::unauthorized();
    }

    EventArtist::clear_all_from_event(parameters.id, connection)?;

    let mut rank = 0;
    let mut added_artists: Vec<EventArtist> = Vec::new();

    for a in &artists.into_inner() {
        added_artists.push(
            EventArtist::create(parameters.id, a.artist_id, rank, a.set_time).commit(connection)?,
        );
        rank += 1;
    }

    Ok(HttpResponse::Ok().json(&added_artists))
}

#[derive(Deserialize)]
pub struct CreateTicketPricingRequest {
    pub name: String,
    pub price_in_cents: i64,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
}

#[derive(Deserialize)]
pub struct CreateTicketTypeRequest {
    pub name: String,
    pub capacity: u32,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub ticket_pricing: Vec<CreateTicketPricingRequest>,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateTicketPricingRequest {
    pub id: Option<Uuid>,
    pub name: Option<String>,
    pub start_date: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,
    pub price_in_cents: Option<i64>,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateTicketTypeRequest {
    //pub id: Uuid,
    pub name: Option<String>,
    pub capacity: Option<u32>,
    pub start_date: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,
    pub ticket_pricing: Option<Vec<UpdateTicketPricingRequest>>,
}

#[derive(Serialize, Deserialize)]
pub struct DisplayCreatedTicket {
    pub id: Uuid,
}

pub fn create_tickets(
    (connection, path, data, user): (
        Connection,
        Path<PathParameters>,
        Json<CreateTicketTypeRequest>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let event = Event::find(path.id, connection)?;
    if !user.has_scope(
        Scopes::EventWrite,
        Some(&event.organization(connection)?),
        connection,
    )? {
        return application::unauthorized();
    }
    //Add new ticket type
    let ticket_type = event.add_ticket_type(
        data.name.clone(),
        data.capacity,
        data.start_date,
        data.end_date,
        connection,
    )?;
    //Add each ticket pricing entry for newly created ticket type
    for curr_pricing_entry in &data.ticket_pricing {
        let _pricing_result = ticket_type.add_ticket_pricing(
            curr_pricing_entry.name.clone(),
            curr_pricing_entry.start_date,
            curr_pricing_entry.end_date,
            curr_pricing_entry.price_in_cents,
            connection,
        )?;
    }
    //    let mut allocation =
    //        TicketAllocation::create(path.id, data.tickets_delta).commit(connection)?;
    //
    //    // TODO: move this to an async processor...
    //    let tari_client = state.get_tari_client();
    //
    //    let asset_id = match tari_client.create_asset(Asset {
    //        id: data.name.clone(),
    //        name: data.name.clone(),
    //        symbol: "sym".into(), //TODO remove symbol from asset spec
    //        decimals: 0,
    //        total_supply: data.tickets_delta,
    //        authorised_signers: vec!["896asudh9872ty4".into()], //TODO add bn-api pub key here
    //        issuer: "BigNeonAddress".into(),
    //        valid: true,
    //        rule_flags: 0,
    //        rule_metadata: "".into(),
    //        expire_date: 10,
    //    }) {
    //        Ok(a) => a,
    //        Err(e) => {
    //            return application::internal_server_error(&format!(
    //                "Could not create tari asset:{}",
    //                e.to_string()
    //            ))
    //        }
    //    };
    //
    //    allocation.set_asset_id(asset_id);
    //
    //    let updated_allocation = allocation.update(connection)?;
    Ok(HttpResponse::Created().json(DisplayCreatedTicket { id: ticket_type.id }))
}

#[derive(Deserialize, Serialize)]
pub struct TicketTypesResponse {
    pub ticket_types: Vec<AdminDisplayTicketType>,
}

pub fn list_ticket_types(
    (connection, path, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let event = Event::find(path.id, connection)?;
    if !user.has_scope(
        Scopes::EventWrite,
        Some(&event.organization(connection)?),
        connection,
    )? {
        return application::unauthorized();
    }

    let ticket_types = TicketType::find_by_event_id(path.id, connection)?;
    let mut encoded_ticket_types = Vec::new();
    for t in ticket_types {
        encoded_ticket_types.push(AdminDisplayTicketType::from_ticket_type(&t, connection)?);
    }

    Ok(HttpResponse::Ok().json(TicketTypesResponse {
        ticket_types: encoded_ticket_types,
    }))
}

pub fn update_tickets(
    (connection, path, data, user): (
        Connection,
        Path<EventTicketPathParameters>,
        Json<UpdateTicketTypeRequest>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let event = Event::find(path.event_id, connection)?;
    if !user.has_scope(
        Scopes::EventWrite,
        Some(&event.organization(connection)?),
        connection,
    )? {
        return application::unauthorized();
    }

    //Update the editable attributes of the ticket type
    let update_parameters = TicketTypeEditableAttributes {
        name: data.name.clone(),
        start_date: data.start_date.clone(),
        end_date: data.end_date.clone(),
    };
    let ticket_type = TicketType::find(path.ticket_type_id, connection)?;
    let updated_ticket_type = ticket_type.update(update_parameters, connection)?;

    if data.ticket_pricing.is_some() {
        let data_ticket_pricings = data.into_inner().ticket_pricing.unwrap();
        //Retrieve the current list of pricings associated with this ticket_type and remove unwanted pricings
        let ticket_pricings = updated_ticket_type.ticket_pricing(connection)?;
        for curr_ticket_pricing in &ticket_pricings {
            let mut found_flag = false;
            for request_ticket_pricing in &data_ticket_pricings {
                if request_ticket_pricing.id.is_some() {
                    if curr_ticket_pricing.id == request_ticket_pricing.id.unwrap() {
                        found_flag = true;
                        break;
                    }
                }
            }
            if !found_flag {
                curr_ticket_pricing.destroy(connection)?;
            }
        }

        //Update the editable attributes for remaining ticket pricings
        for curr_ticket_pricing in &data_ticket_pricings {
            if curr_ticket_pricing.id.is_some() {
                //Update the ticket pricing
                let update_parameters = TicketPricingEditableAttributes {
                    name: curr_ticket_pricing.name.clone(),
                    price_in_cents: curr_ticket_pricing.price_in_cents,
                    start_date: curr_ticket_pricing.start_date,
                    end_date: curr_ticket_pricing.end_date,
                };
                let curr_ticket_pricing_id = curr_ticket_pricing.id.unwrap();
                let found_index = ticket_pricings
                    .iter()
                    .position(|ref r| r.id == curr_ticket_pricing_id);
                if found_index.is_some() {
                    ticket_pricings[found_index.unwrap()].update(update_parameters, connection)?;
                }
            } else {
                //Only create a new pricing entry if all of its required data was provided
                if curr_ticket_pricing.name.is_some()
                    && curr_ticket_pricing.price_in_cents.is_some()
                    && curr_ticket_pricing.start_date.is_some()
                    && curr_ticket_pricing.end_date.is_some()
                {
                    let curr_ticket_pricing_name = curr_ticket_pricing.name.clone().unwrap();

                    //Add new ticket pricing
                    let _pricing_result = updated_ticket_type.add_ticket_pricing(
                        curr_ticket_pricing_name,
                        curr_ticket_pricing.start_date.unwrap(),
                        curr_ticket_pricing.end_date.unwrap(),
                        curr_ticket_pricing.price_in_cents.unwrap(),
                        connection,
                    )?;
                } else {
                    //TODO send error when all data was not specified

                }
            }
        }
    }

    Ok(HttpResponse::Ok().finish())
}
