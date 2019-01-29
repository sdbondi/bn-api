use chrono::NaiveDateTime;
use diesel;
use diesel::expression::dsl;
use diesel::prelude::*;
use models::users::User;
use models::*;
use schema::{organization_users, organizations, venues};
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;
use validator::Validate;

#[derive(
    Clone,
    Associations,
    Identifiable,
    Queryable,
    AsChangeset,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
)]
#[belongs_to(Region)]
#[table_name = "venues"]
pub struct Venue {
    pub id: Uuid,
    pub region_id: Uuid,
    pub organization_id: Option<Uuid>,
    pub is_private: bool,
    pub name: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub country: String,
    pub postal_code: String,
    pub phone: Option<String>,
    pub promo_image_url: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub google_place_id: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: String,
}

#[derive(AsChangeset, Default, Deserialize, Validate)]
#[table_name = "venues"]
pub struct VenueEditableAttributes {
    pub region_id: Option<Uuid>,
    pub name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub address: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub city: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub state: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub country: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub postal_code: Option<String>,
    #[serde(default, deserialize_with = "double_option_deserialize_unless_blank")]
    pub phone: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option_deserialize_unless_blank")]
    pub promo_image_url: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option_deserialize_unless_blank")]
    pub google_place_id: Option<Option<String>>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    #[validate(length(min = "1", message = "Timezone is invalid"))]
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub timezone: Option<String>,
}

#[derive(Default, Insertable, Serialize, Deserialize, PartialEq, Debug, Clone, Validate)]
#[table_name = "venues"]
pub struct NewVenue {
    pub name: String,
    pub region_id: Option<Uuid>,
    pub organization_id: Option<Uuid>,
    pub address: String,
    pub city: String,
    pub state: String,
    pub country: String,
    pub postal_code: String,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub phone: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub promo_image_url: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub google_place_id: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    #[validate(length(min = "1", message = "Timezone is invalid"))]
    pub timezone: String,
}

impl NewVenue {
    pub fn commit(&self, connection: &PgConnection) -> Result<Venue, DatabaseError> {
        let mut record = self.clone();

        if self.region_id.is_none() {
            let region = match Region::find_by_name(&self.state, connection)? {
                Some(r) => r,
                None => Region::create(self.state.clone()).commit(connection)?,
            };
            record.region_id = Some(region.id);
        }
        record.validate()?;

        DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not create new venue",
            diesel::insert_into(venues::table)
                .values(record)
                .get_result(connection),
        )
    }
}

impl Venue {
    pub fn create(
        name: &str,
        region_id: Option<Uuid>,
        organization_id: Option<Uuid>,
        timezone: String,
    ) -> NewVenue {
        NewVenue {
            name: String::from(name),
            region_id,
            organization_id,
            timezone,
            ..Default::default()
        }
    }

    pub fn update(
        &self,
        attributes: VenueEditableAttributes,
        conn: &PgConnection,
    ) -> Result<Venue, DatabaseError> {
        attributes.validate()?;
        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Could not update venue",
            diesel::update(self)
                .set((attributes, venues::updated_at.eq(dsl::now)))
                .get_result(conn),
        )
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<Venue, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading venue",
            venues::table.find(id).first::<Venue>(conn),
        )
    }

    pub fn find_by_ids(
        venue_ids: Vec<Uuid>,
        conn: &PgConnection,
    ) -> Result<Vec<Venue>, DatabaseError> {
        venues::table
            .filter(venues::id.eq_any(venue_ids))
            .order_by(venues::name)
            .select(venues::all_columns)
            .order_by(venues::id.asc())
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Unable to load venues by ids")
    }

    pub fn all(user: Option<&User>, conn: &PgConnection) -> Result<Vec<Venue>, DatabaseError> {
        let query = match user {
            Some(u) => venues::table
                .left_join(
                    organization_users::table.on(venues::organization_id
                        .eq(organization_users::organization_id.nullable())
                        .and(organization_users::user_id.eq(u.id))),
                )
                .left_join(
                    organizations::table
                        .on(venues::organization_id.eq(organizations::id.nullable())),
                )
                .filter(
                    organization_users::user_id
                        .eq(u.id)
                        .or(venues::is_private.eq(false))
                        .or(dsl::sql("TRUE = ").bind::<diesel::sql_types::Bool, _>(u.is_admin())),
                )
                .order_by(venues::name)
                .select(venues::all_columns)
                .load(conn),
            None => venues::table
                .filter(venues::is_private.eq(false))
                .order_by(venues::name)
                .select(venues::all_columns)
                .load(conn),
        };

        query.to_db_error(ErrorCode::QueryError, "Unable to load all venues")
    }

    pub fn find_for_organization(
        user_id: Option<Uuid>,
        organization_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Vec<Venue>, DatabaseError> {
        let query = match user_id {
            Some(u) => venues::table
                .left_join(
                    organization_users::table.on(venues::organization_id
                        .eq(organization_users::organization_id.nullable())
                        .and(organization_users::user_id.eq(u))),
                )
                .left_join(
                    organizations::table
                        .on(venues::organization_id.eq(organizations::id.nullable())),
                )
                .filter(
                    organization_users::user_id
                        .eq(u)
                        .or(venues::is_private.eq(false)),
                )
                .filter(venues::organization_id.eq(organization_id))
                .order_by(venues::name)
                .select(venues::all_columns)
                .load(conn),
            None => venues::table
                .filter(venues::is_private.eq(false))
                .filter(venues::organization_id.eq(organization_id))
                .order_by(venues::name)
                .select(venues::all_columns)
                .load(conn),
        };

        query.to_db_error(ErrorCode::QueryError, "Unable to load all venues")
    }

    pub fn add_to_organization(
        self,
        organization_id: &Uuid,
        conn: &PgConnection,
    ) -> Result<Venue, DatabaseError> {
        //Should I make sure that this venue doesn't already have one here even though there is logic
        //for that in the bn-api layer?
        diesel::update(&self)
            .set(venues::organization_id.eq(organization_id))
            .get_result(conn)
            .to_db_error(ErrorCode::UpdateError, "Could not update venue")
    }

    pub fn set_privacy(
        &self,
        is_private: bool,
        conn: &PgConnection,
    ) -> Result<Venue, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Could not update is_private for artist",
            diesel::update(self)
                .set((
                    venues::is_private.eq(is_private),
                    venues::updated_at.eq(dsl::now),
                ))
                .get_result(conn),
        )
    }

    pub fn organization(&self, conn: &PgConnection) -> Result<Option<Organization>, DatabaseError> {
        match self.organization_id {
            Some(organization_id) => Ok(Some(Organization::find(organization_id, conn)?)),
            None => Ok(None),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DisplayVenue {
    pub id: Uuid,
    pub name: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub country: String,
    pub postal_code: String,
    pub phone: Option<String>,
    pub promo_image_url: Option<String>,
}

impl From<Venue> for DisplayVenue {
    fn from(venue: Venue) -> Self {
        DisplayVenue {
            id: venue.id,
            name: venue.name,
            address: venue.address,
            city: venue.city,
            state: venue.state,
            country: venue.country,
            postal_code: venue.postal_code,
            phone: venue.phone,
            promo_image_url: venue.promo_image_url,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VenueInfo {
    pub id: Uuid,
    pub name: String,
    pub timezone: String,
}
