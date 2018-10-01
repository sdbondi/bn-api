use chrono::NaiveDate;
use chrono::NaiveDateTime;
use diesel;
use diesel::expression::dsl;
use diesel::prelude::*;
use models::*;
use schema::{artists, event_artists, events, venues};
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use utils::errors::*;
use uuid::Uuid;
use validator::Validate;

#[derive(Associations, Identifiable, Queryable, AsChangeset)]
#[belongs_to(Organization)]
#[derive(Clone, QueryableByName, Serialize, Deserialize, PartialEq, Debug)]
#[belongs_to(Venue)]
#[table_name = "events"]
pub struct Event {
    pub id: Uuid,
    pub name: String,
    pub organization_id: Uuid,
    pub venue_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
    pub event_start: Option<NaiveDateTime>,
    pub door_time: Option<NaiveDateTime>,
    pub status: String,
    pub publish_date: Option<NaiveDateTime>,
    pub promo_image_url: Option<String>,
    pub additional_info: Option<String>,
    pub age_limit: Option<i32>,
    pub cancelled_at: Option<NaiveDateTime>,
    pub updated_at: NaiveDateTime,
}

#[derive(Default, Insertable, Serialize, Deserialize, Validate)]
#[table_name = "events"]
pub struct NewEvent {
    pub name: String,
    pub organization_id: Uuid,
    pub venue_id: Option<Uuid>,
    pub event_start: Option<NaiveDateTime>,
    pub door_time: Option<NaiveDateTime>,
    #[serde(default = "NewEvent::default_status", skip_deserializing)]
    pub status: String,
    pub publish_date: Option<NaiveDateTime>,
    #[validate(url)]
    pub promo_image_url: Option<String>,
    pub additional_info: Option<String>,
    pub age_limit: Option<i32>,
}

impl NewEvent {
    pub fn commit(&self, conn: &PgConnection) -> Result<Event, DatabaseError> {
        diesel::insert_into(events::table)
            .values(self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create new event")
    }

    pub fn default_status() -> String {
        EventStatus::Draft.to_string()
    }
}

#[derive(AsChangeset, Default, Deserialize, Validate)]
#[table_name = "events"]
pub struct EventEditableAttributes {
    pub name: Option<String>,
    pub venue_id: Option<Uuid>,
    pub event_start: Option<NaiveDateTime>,
    pub door_time: Option<NaiveDateTime>,
    pub publish_date: Option<NaiveDateTime>,
    #[validate(url)]
    pub promo_image_url: Option<String>,
    pub additional_info: Option<String>,
    pub age_limit: Option<i32>,
    pub cancelled_at: Option<NaiveDateTime>,
}

impl Event {
    pub fn create(
        name: &str,
        status: EventStatus,
        organization_id: Uuid,
        venue_id: Option<Uuid>,
        event_start: Option<NaiveDateTime>,
        door_time: Option<NaiveDateTime>,
        publish_date: Option<NaiveDateTime>,
    ) -> NewEvent {
        NewEvent {
            name: name.into(),
            status: status.to_string(),
            organization_id,
            venue_id,
            event_start,
            door_time,
            publish_date,
            ..Default::default()
        }
    }

    pub fn update(
        &self,
        attributes: EventEditableAttributes,
        conn: &PgConnection,
    ) -> Result<Event, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Could not update event",
            diesel::update(self)
                .set((attributes, events::updated_at.eq(dsl::now)))
                .get_result(conn),
        )
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<Event, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading event",
            events::table.find(id).first::<Event>(conn),
        )
    }

    pub fn cancel(self, conn: &PgConnection) -> Result<Event, DatabaseError> {
        diesel::update(&self)
            .set(events::cancelled_at.eq(dsl::now.nullable()))
            .get_result(conn)
            .to_db_error(ErrorCode::UpdateError, "Could not update event")
    }

    pub fn find_all_events_from_venue(
        venue_id: &Uuid,
        conn: &PgConnection,
    ) -> Result<Vec<Event>, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading event via venue",
            events::table
                .filter(events::venue_id.eq(venue_id))
                .load(conn),
        )
    }

    pub fn find_all_events_from_organization(
        organization_id: &Uuid,
        conn: &PgConnection,
    ) -> Result<Vec<Event>, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading events via organization",
            events::table
                .filter(events::organization_id.eq(organization_id))
                .load(conn),
        )
    }

    pub fn search(
        query_filter: Option<String>,
        region_id: Option<Uuid>,
        start_time: Option<NaiveDateTime>,
        end_time: Option<NaiveDateTime>,
        status_filter: Option<Vec<EventStatus>>,
        conn: &PgConnection,
    ) -> Result<Vec<Event>, DatabaseError> {
        let query_like = match query_filter {
            Some(n) => format!("%{}%", n),
            None => "%".to_string(),
        };
        let mut query = events::table
            .filter(
                events::event_start
                    .gt(start_time
                        .unwrap_or_else(|| NaiveDate::from_ymd(1970, 1, 1).and_hms(0, 0, 0))),
            ).filter(
                events::event_start
                    .lt(end_time
                        .unwrap_or_else(|| NaiveDate::from_ymd(3970, 1, 1).and_hms(0, 0, 0))),
            ).left_join(venues::table.on(events::venue_id.eq(venues::id.nullable())))
            .left_join(
                event_artists::table
                    .inner_join(
                        artists::table.on(event_artists::artist_id
                            .eq(artists::id)
                            .and(artists::name.ilike(query_like.clone()))),
                    ).on(events::id.eq(event_artists::event_id)),
            ).filter(
                events::name
                    .ilike(query_like.clone())
                    .or(venues::id
                        .is_not_null()
                        .and(venues::name.ilike(query_like.clone()))).or(artists::id.is_not_null()),
            ).filter(events::status.ne(EventStatus::Draft.to_string()))
            .select(events::all_columns)
            .distinct()
            .order_by(events::event_start.asc())
            .then_order_by(events::name.asc())
            .into_boxed();

        if let Some(statuses) = status_filter {
            let statuses: Vec<String> = statuses
                .into_iter()
                .map(|status| status.to_string())
                .collect();
            query = query.filter(events::status.eq_any(statuses));
        }

        if let Some(region_id) = region_id {
            query = query.filter(venues::region_id.eq(region_id));
        }

        let result = query.load(conn);

        DatabaseError::wrap(ErrorCode::QueryError, "Unable to load all events", result)
    }

    pub fn add_artist(&self, artist_id: Uuid, conn: &PgConnection) -> Result<(), DatabaseError> {
        EventArtist::create(self.id, artist_id, 0, None)
            .commit(conn)
            .map(|_| ())
    }

    pub fn organization(&self, conn: &PgConnection) -> Result<Organization, DatabaseError> {
        Organization::find(self.organization_id, conn)
    }

    pub fn venue(&self, conn: &PgConnection) -> Result<Option<Venue>, DatabaseError> {
        match self.venue_id {
            Some(venue_id) => {
                let venue = Venue::find(venue_id, conn);
                match venue {
                    Ok(venue) => Ok(Some(venue)),
                    Err(e) => Err(e),
                }
            }
            None => Ok(None),
        }
    }

    pub fn add_ticket_type(
        &self,
        name: String,
        quantity: u32,
        start_date: NaiveDateTime,
        end_date: NaiveDateTime,
        conn: &PgConnection,
    ) -> Result<TicketType, DatabaseError> {
        let ticket_type =
            TicketType::create(self.id, name.clone(), start_date, end_date).commit(conn)?;
        let asset =
            Asset::create(ticket_type.id, format!("{}.{}", self.name, &name)).commit(conn)?;
        TicketInstance::create_multiple(asset.id, quantity, conn)?;
        Ok(ticket_type)
    }

    pub fn ticket_types(&self, conn: &PgConnection) -> Result<Vec<TicketType>, DatabaseError> {
        TicketType::find_by_event_id(self.id, conn)
    }

    pub fn for_display(self, conn: &PgConnection) -> Result<DisplayEvent, DatabaseError> {
        let venue: Option<DisplayVenue> =
            self.venue(conn)?.map_or(None, |venue| Some(venue.into()));

        Ok(DisplayEvent {
            id: self.id,
            name: self.name,
            event_start: self.event_start,
            door_time: self.door_time,
            promo_image_url: self.promo_image_url,
            additional_info: self.additional_info,
            venue,
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DisplayEvent {
    pub id: Uuid,
    pub name: String,
    pub event_start: Option<NaiveDateTime>,
    pub door_time: Option<NaiveDateTime>,
    pub promo_image_url: Option<String>,
    pub additional_info: Option<String>,
    pub venue: Option<DisplayVenue>,
}
