use actix_web::error;
use actix_web::middleware::Finished;
use actix_web::middleware::Logger;
use actix_web::middleware::Middleware;
use actix_web::middleware::Started;
use actix_web::FromRequest;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use extractors::OptionalUser;
use log::Level;
use server::AppState;

pub struct BigNeonLogger {
    logger: Logger,
}

impl BigNeonLogger {
    pub fn new(format: &str) -> BigNeonLogger {
        BigNeonLogger {
            logger: Logger::new(format),
        }
    }
}

impl Middleware<AppState> for BigNeonLogger {
    fn start(&self, req: &HttpRequest<AppState>) -> error::Result<Started> {
        self.logger.start(req)?;
        let user = OptionalUser::from_request(req, &());
        let ip_address = req.connection_info().remote().map(|i| i.to_string());
        let uri = req.uri().to_string();
        let method = req.method().to_string();
        jlog!(
            Level::Info,
            "bigneon_api::big_neon_logger",
            format!("{} {} starting", method, uri).as_str(),
            {
                "user_id": user.ok().map(|u| u.0.map(|v| v.id())),
                "ip_address": ip_address,
                "uri": uri,
                "method": method,
                "api_version": env!("CARGO_PKG_VERSION")
        });
        Ok(Started::Done)
    }

    fn finish(&self, req: &HttpRequest<AppState>, resp: &HttpResponse) -> Finished {
        match resp.error() {
            Some(error) => {
                let user = OptionalUser::from_request(req, &());
                let ip_address = req.connection_info().remote().map(|i| i.to_string());
                let uri = req.uri().to_string();
                let method = req.method().to_string();
                jlog!(
                    Level::Error,
                    "bigneon_api::big_neon_logger",
                    &error.to_string(),
                    {
                        "user_id": user.ok().map(|u| u.0.map(|v| v.id())),
                        "ip_address": ip_address,
                        "uri": uri,
                        "method": method,
                        "api_version": env!("CARGO_PKG_VERSION")
                });
                Finished::Done
            }
            None => self.logger.finish(req, resp),
        }
    }
}
