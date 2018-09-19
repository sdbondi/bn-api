use actix_web::{http::StatusCode, FromRequest, HttpResponse, Json, Path, Query};
use bigneon_api::controllers::events;
use bigneon_api::controllers::events::*;
use bigneon_db::models::*;
use chrono::prelude::*;
use diesel::PgConnection;
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;
use uuid::Uuid;

pub fn create(role: Roles, should_test_succeed: bool, same_organization: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = if same_organization && role != Roles::User {
        database.create_organization_with_user(&user, role == Roles::OrgOwner)
    } else {
        database.create_organization()
    }.finish();

    let venue = database.create_venue().finish();
    let auth_user = support::create_auth_user_from_user(&user, role, &database);

    let name = "event Example";
    let new_event = CreateEventRequest {
        name: name.clone().to_string(),
        organization_id: organization.id,
        venue_id: Some(venue.id),
        event_start: Some(NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11)),
        door_time: Some(NaiveDate::from_ymd(2016, 7, 8).and_hms(8, 11, 12)),
        publish_date: Some(NaiveDate::from_ymd(2016, 7, 1).and_hms(9, 10, 11)),
        additional_info: None,
        age_limit: None,
        promo_image_url: None,
    };
    let json = Json(new_event);

    let response: HttpResponse =
        events::create((database.connection.into(), json, auth_user.clone())).into();
    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::CREATED);
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn update(role: Roles, should_test_succeed: bool, same_organization: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = if same_organization && role != Roles::User {
        database.create_organization_with_user(&user, role == Roles::OrgOwner)
    } else {
        database.create_organization()
    }.finish();
    let auth_user = support::create_auth_user_from_user(&user, role, &database);
    let event = database
        .create_event()
        .with_organization(&organization)
        .finish();

    let new_name = "New Event Name";
    let test_request = TestRequest::create();

    let json = Json(EventEditableAttributes {
        name: Some(new_name.clone().to_string()),
        ..Default::default()
    });
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event.id;

    let response: HttpResponse =
        events::update((database.connection.into(), path, json, auth_user.clone())).into();
    let body = support::unwrap_body_to_string(&response).unwrap();
    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        let updated_event: Event = serde_json::from_str(&body).unwrap();
        assert_eq!(updated_event.name, new_name);
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn show(role: Roles) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let auth_user = support::create_auth_user_from_user(&user, role, &database);

    let organization = if role != Roles::User {
        database.create_organization_with_user(&user, role == Roles::OrgOwner)
    } else {
        database.create_organization()
    }.finish();

    let venue = database.create_venue().finish();
    let event = database
        .create_event()
        .with_name("NewEvent".to_string())
        .with_organization(&organization)
        .with_venue(&venue)
        .finish();
    let event_id = event.id;

    let artist1 = database.create_artist().finish();
    let artist2 = database.create_artist().finish();

    event.add_artist(artist1.id, &database.connection).unwrap();
    event.add_artist(artist2.id, &database.connection).unwrap();

    let _event_interest = EventInterest::create(event.id, user.id).commit(&database.connection);
    let event_expected_json = expected_show_json(event, organization, venue, &database.connection);

    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event_id;

    let response: HttpResponse =
        events::show((database.connection.into(), path, Some(auth_user))).into();
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body, event_expected_json);
}

pub fn cancel(role: Roles, should_test_succeed: bool, same_organization: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = if same_organization && role != Roles::User {
        database.create_organization_with_user(&user, role == Roles::OrgOwner)
    } else {
        database.create_organization()
    }.finish();

    let event = database
        .create_event()
        .with_organization(&organization)
        .finish();
    let auth_user = support::create_auth_user_from_user(&user, role, &database);
    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event.id;

    let response: HttpResponse =
        events::cancel((database.connection.into(), path, auth_user.clone())).into();
    let body = support::unwrap_body_to_string(&response).unwrap();
    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        let updated_event: Event = serde_json::from_str(&body).unwrap();
        assert!(!updated_event.cancelled_at.is_none());
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn add_artist(role: Roles, should_test_succeed: bool, same_organization: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let auth_user = support::create_auth_user_from_user(&user, role, &database);
    let organization = if same_organization && role != Roles::User {
        database.create_organization_with_user(&user, role == Roles::OrgOwner)
    } else {
        database.create_organization()
    }.finish();

    let event = database
        .create_event()
        .with_organization(&organization)
        .finish();
    let artist = database
        .create_artist()
        .with_organization(&organization)
        .finish();

    let test_request = TestRequest::create();

    let new_event_artist = AddArtistRequest {
        artist_id: artist.id,
        rank: 5,
        set_time: Some(NaiveDate::from_ymd(2016, 7, 1).and_hms(9, 10, 11)),
    };

    let json = Json(new_event_artist);

    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event.id;

    let response: HttpResponse =
        events::add_artist((database.connection.into(), path, json, auth_user.clone())).into();
    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::CREATED);
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn list_interested_users(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let event = database.create_event().finish();
    let primary_user = support::create_auth_user(role, &database);
    EventInterest::create(event.id, primary_user.id())
        .commit(&database.connection)
        .unwrap();
    let n_secondary_users = 5;
    let mut secondary_users: Vec<DisplayEventInterestedUser> = Vec::new();
    secondary_users.reserve(n_secondary_users);
    for _u_id in 0..n_secondary_users {
        let curr_secondary_user = database.create_user().finish();
        EventInterest::create(event.id, curr_secondary_user.id)
            .commit(&database.connection)
            .unwrap();
        let curr_user_entry = DisplayEventInterestedUser {
            user_id: curr_secondary_user.id,
            first_name: curr_secondary_user.first_name.clone(),
            last_name: curr_secondary_user.last_name.clone(),
            thumb_profile_pic_url: None,
        };
        secondary_users.push(curr_user_entry);
    }
    secondary_users.sort_by_key(|x| x.user_id); //Sort results for testing purposes
                                                //Construct api query
    let from_index: usize = 0;
    let to_index: usize = 10;
    let test_request = TestRequest::create_with_uri(&format!(
        "/interest?from_index={}&to_index={}",
        from_index.to_string(),
        to_index.to_string()
    ));
    let query_parameters =
        Query::<PagingSearchParameters>::from_request(&test_request.request, &()).unwrap();
    let mut path_parameters = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path_parameters.id = event.id;
    let response: HttpResponse = events::list_interested_users((
        database.connection.into(),
        path_parameters,
        query_parameters,
        primary_user,
    )).into();
    let response_body = support::unwrap_body_to_string(&response).unwrap();
    //Construct expected output
    let expected_data = DisplayEventInterestedUserList {
        total_interests: secondary_users.len(),
        users: secondary_users,
    };
    let expected_json_body = serde_json::to_string(&expected_data).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response_body, expected_json_body);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let updated_event = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(response_body, updated_event);
    }
}

pub fn add_interest(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let event = database.create_event().finish();

    let user = support::create_auth_user(role, &database);
    let test_request = TestRequest::create();

    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event.id;

    let response: HttpResponse =
        events::add_interest((database.connection.into(), path, user)).into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::CREATED);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let updated_event = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, updated_event);
    }
}

pub fn remove_interest(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let event = database.create_event().finish();
    EventInterest::create(event.id, user.id)
        .commit(&database.connection)
        .unwrap();

    let user = support::create_auth_user_from_user(&user, role, &database);
    let test_request = TestRequest::create();

    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event.id;

    let response: HttpResponse =
        events::remove_interest((database.connection.into(), path, user)).into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body, "1");
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let updated_event = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, updated_event);
    }
}

pub fn update_artists(role: Roles, should_test_succeed: bool, same_organization: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let auth_user = support::create_auth_user_from_user(&user, role, &database);
    let organization = if same_organization && role != Roles::User {
        database.create_organization_with_user(&user, role == Roles::OrgOwner)
    } else {
        database.create_organization()
    }.finish();

    let event = database
        .create_event()
        .with_organization(&organization)
        .finish();
    let artist1 = database.create_artist().finish();
    let artist2 = database.create_artist().finish();
    let test_request = TestRequest::create();

    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event.id;

    let mut payload: Vec<UpdateArtistsRequest> = Vec::new();
    payload.push(UpdateArtistsRequest {
        artist_id: artist1.id,
        set_time: Some(NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11)),
    });
    payload.push(UpdateArtistsRequest {
        artist_id: artist2.id,
        set_time: None,
    });

    let response: HttpResponse = events::update_artists((
        database.connection.into(),
        path,
        Json(payload),
        auth_user.clone(),
    )).into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        let returned_event_artists: Vec<EventArtist> = serde_json::from_str(&body).unwrap();
        assert_eq!(returned_event_artists[0].artist_id, artist1.id);
        assert_eq!(returned_event_artists[1].set_time, None);
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn create_tickets(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = if role != Roles::User {
        database.create_organization_with_user(&user, role == Roles::OrgOwner)
    } else {
        database.create_organization()
    }.finish();
    let event = database
        .create_event()
        .with_organization(&organization)
        .finish();
    let user = support::create_auth_user_from_user(&user, role, &database);
    //Construct Ticket creation and pricing request
    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event.id;
    let mut ticket_pricing: Vec<CreateTicketPricingRequest> = Vec::new();
    let start_date = NaiveDate::from_ymd(2018, 5, 1).and_hms(6, 20, 21);
    let middle_date = NaiveDate::from_ymd(2018, 6, 2).and_hms(7, 45, 31);
    let end_date = NaiveDate::from_ymd(2018, 7, 3).and_hms(9, 23, 23);
    ticket_pricing.push(CreateTicketPricingRequest {
        name: String::from("Early bird"),
        price_in_cents: 10000,
        start_date,
        end_date: middle_date,
    });
    ticket_pricing.push(CreateTicketPricingRequest {
        name: String::from("Base"),
        price_in_cents: 20000,
        start_date: middle_date,
        end_date,
    });
    let request_data = CreateTicketTypeRequest {
        name: "VIP".into(),
        capacity: 1000,
        start_date,
        end_date,
        ticket_pricing,
    };
    let response: HttpResponse =
        events::create_tickets((database.connection.into(), path, Json(request_data), user)).into();

    let body = support::unwrap_body_to_string(&response).unwrap();
    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::CREATED);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let updated_event = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, updated_event);
    }
}

fn expected_show_json(
    event: Event,
    organization: Organization,
    venue: Venue,
    connection: &PgConnection,
) -> String {
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
        venue: Venue,
        artists: Vec<DisplayEventArtist>,
        total_interest: u32,
        user_is_interested: bool,
    }

    let event_artists = EventArtist::find_all_from_event(event.id, connection).unwrap();

    let display_event_artists: Vec<DisplayEventArtist> = event_artists
        .iter()
        .map(|e| DisplayEventArtist {
            event_id: e.event_id,
            artist_id: e.artist_id,
            rank: e.rank,
            set_time: e.set_time,
        }).collect();

    serde_json::to_string(&R {
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
        total_interest: 1,
        user_is_interested: true,
    }).unwrap()
}
