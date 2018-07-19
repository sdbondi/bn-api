use actix_web::middleware::session::{CookieSessionBackend, SessionStorage};
use actix_web::{server, App};
use config::Config;
use database::{ConnectionGranting, Database};
use routing;

pub struct AppState {
    pub database: Box<ConnectionGranting>,
}

pub struct Server {
    pub config: Config,
}

impl Server {
    pub fn start(config: Config) {
        server::new({
            move || {
                routing::route(App::with_state(AppState {
                    database: Box::new(Database::from_config(&config)),
                })).middleware(SessionStorage::new(
                    CookieSessionBackend::private(config.cookie_secret_key.as_bytes())
                        .secure(false),
                ))
            }
        }).bind("127.0.0.1:8088")
            .expect("Can not bind to 127.0.0.1:8088")
            .run();
    }
}
