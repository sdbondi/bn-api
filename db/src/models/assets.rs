use chrono::prelude::*;
use diesel;
use diesel::expression::dsl;
use diesel::prelude::*;
use models::*;
use schema::assets;
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Queryable, Identifiable, AsChangeset, Debug)]
#[table_name = "assets"]
pub struct Asset {
    pub id: Uuid,
    ticket_type_id: Uuid,
    blockchain_name: String,
    // TODO: This will be populated after it is created on the blockchain.
    pub blockchain_asset_id: Option<String>,
    status: AssetStatus,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl Asset {
    pub fn create(ticket_type_id: Uuid, blockchain_name: String) -> NewAsset {
        let random_uuid = Uuid::new_v4();
        let blockchain_name = format!("{}_{}", blockchain_name, random_uuid.to_string());
        NewAsset {
            blockchain_name,
            ticket_type_id,
            status: AssetStatus::Unsynced,
        }
    }

    pub fn find_by_ticket_type(
        ticket_type_id: &Uuid,
        conn: &PgConnection,
    ) -> Result<Asset, DatabaseError> {
        assets::table
            .filter(assets::ticket_type_id.eq(ticket_type_id))
            .first(conn)
            .to_db_error(ErrorCode::QueryError, "Error loading asset")
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<Asset, DatabaseError> {
        assets::table
            .find(id)
            .first::<Asset>(conn)
            .to_db_error(ErrorCode::QueryError, "Error loading asset")
    }

    pub fn update_blockchain_id(
        &self,
        id: String,
        conn: &PgConnection,
    ) -> Result<Asset, DatabaseError> {
        diesel::update(self)
            .set((
                assets::blockchain_asset_id.eq(id),
                assets::updated_at.eq(dsl::now),
            ))
            .get_result(conn)
            .to_db_error(
                ErrorCode::UpdateError,
                "Could not update asset blockchain id",
            )
    }
}

#[derive(Insertable)]
#[table_name = "assets"]
pub struct NewAsset {
    blockchain_name: String,
    status: AssetStatus,
    ticket_type_id: Uuid,
}

impl NewAsset {
    pub fn commit(self, conn: &PgConnection) -> Result<Asset, DatabaseError> {
        diesel::insert_into(assets::table)
            .values(self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create asset")
    }
}
