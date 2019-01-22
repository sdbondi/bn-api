use actix_web::{http::StatusCode, HttpResponse, Path, Query};
use auth::user::User;
use bigneon_db::models::*;
use db::Connection;
use errors::*;
use extractors::*;
use helpers::application;
use models::{CreateArtistRequest, PathParameters, WebPayload};
use utils::spotify;

pub fn search(
    (connection, query_parameters, user): (Connection, Query<PagingParameters>, OptionalUser),
) -> Result<WebPayload<CreateArtistRequest>, BigNeonError> {
    let db_user = user.into_inner().map(|u| u.user);
    let artists = Artist::search(&db_user, query_parameters.get_tag("q"), connection.get())?;

    let try_spotify = query_parameters
        .get_tag("spotify")
        .map(|spotify| spotify != "0")
        .unwrap_or(false);
    if try_spotify && artists.is_empty() && query_parameters.get_tag("q").is_some() {
        //Try spotify
	let spotify_client = &spotify::SINGLETON;
        let spotify_artists =
            spotify_client.search(query_parameters.get_tag("q").unwrap_or("".to_string()))?;

        let payload = Payload::new(spotify_artists, query_parameters.into_inner().into());
        Ok(WebPayload::new(StatusCode::OK, payload))
    } else {
        let wrapper = artists
            .into_iter()
            .map(|a| CreateArtistRequest::from(a))
            .collect();
        let payload = Payload::new(wrapper, query_parameters.into_inner().into());
        Ok(WebPayload::new(StatusCode::OK, payload))
    }
}

pub fn index(
    (connection, query_parameters, user): (Connection, Query<PagingParameters>, OptionalUser),
) -> Result<HttpResponse, BigNeonError> {
    let db_user = user.into_inner().map(|u| u.user);
    let artists = Artist::search(&db_user, query_parameters.get_tag("q"), connection.get())?;
    let payload = Payload::from_data(artists, query_parameters.page(), query_parameters.limit());
    Ok(HttpResponse::Ok().json(&payload))
}

pub fn show(
    (connection, parameters): (Connection, Path<PathParameters>),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let artist = Artist::find(&parameters.id, connection)?;
    Ok(HttpResponse::Ok().json(&artist))
}

pub fn create(
    (connection, json_create_artist, user): (Connection, Json<CreateArtistRequest>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    if let Some(organization_id) = json_create_artist.organization_id {
        let organization = Organization::find(organization_id, connection)?;
        user.requires_scope_for_organization(Scopes::ArtistWrite, &organization, connection)?;
    } else {
        user.requires_scope(Scopes::ArtistWrite)?;
    }

    let create_artist = json_create_artist.into_inner();
    let mut artist = match &create_artist.spotify_id {
        Some(spotify_id) => {
	    let spotify_client = &spotify::SINGLETON;

            let spotify_artist_result = spotify_client.read_artist(&spotify_id)?;
            match spotify_artist_result {
                Some(artist) => {
                    let mut new_artist: NewArtist = artist.into();
                    let client_data: NewArtist = create_artist.clone().into();
                    new_artist.merge(client_data);
                    new_artist.commit(connection)?
                }
                None => return application::not_found(),
            }
        }
        None => {
            let new_artist: NewArtist = create_artist.clone().into();
            new_artist.commit(connection)?
        }
    };

    // New artists belonging to an organization start private
    if artist.organization_id.is_some() {
        artist = artist.set_privacy(true, connection)?;
    }
    Ok(HttpResponse::Created().json(&artist))
}

pub fn show_from_organizations(
    (connection, organization_id, query_parameters, user): (
        Connection,
        Path<PathParameters>,
        Query<PagingParameters>,
        OptionalUser,
    ),
) -> Result<HttpResponse, BigNeonError> {
    //TODO implement proper paging on db
    let artists = match user.into_inner() {
        Some(u) => {
            Artist::find_for_organization(Some(u.id()), organization_id.id, connection.get())?
        }
        None => Artist::find_for_organization(None, organization_id.id, connection.get())?,
    };
    let payload = Payload::from_data(artists, query_parameters.page(), query_parameters.limit());
    Ok(HttpResponse::Ok().json(&payload))
}

pub fn update(
    (connection, parameters, artist_parameters, user): (
        Connection,
        Path<PathParameters>,
        Json<ArtistEditableAttributes>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let artist = Artist::find(&parameters.id, connection)?;
    if !artist.is_private || artist.organization_id.is_none() {
        user.requires_scope(Scopes::ArtistWrite)?;
    } else {
        let organization = artist.organization(connection)?.unwrap();
        user.requires_scope_for_organization(Scopes::ArtistWrite, &organization, connection)?;
    }

    let updated_artist = artist.update(&artist_parameters, connection)?;
    Ok(HttpResponse::Ok().json(&updated_artist))
}

pub fn toggle_privacy(
    (connection, parameters, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    user.requires_scope(Scopes::ArtistWrite)?;

    let connection = connection.get();
    let artist = Artist::find(&parameters.id, connection)?;
    let updated_artist = artist.set_privacy(!artist.is_private, connection)?;
    Ok(HttpResponse::Ok().json(updated_artist))
}
