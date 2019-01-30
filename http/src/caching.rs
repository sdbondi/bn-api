pub use actix_web::http::header::{CacheControl, CacheDirective, ETag, EntityTag};
use actix_web::{dev::HttpResponseBuilder, http::header, HttpRequest, HttpResponse};

pub struct CacheHeaders(pub header::CacheControl, pub Option<ETag>);

pub trait ToETag {
    fn to_etag(&self) -> header::ETag;
}

impl CacheHeaders {
    pub fn cache_control(&self) -> &header::CacheControl {
	&self.0
    }

    pub fn etag(&self) -> &Option<ETag> {
	&self.1
    }

    pub fn into_ok_response(self) -> HttpResponseBuilder {
	let mut builder = HttpResponse::Ok();

	self.set_headers(&mut builder);
	builder
    }

    /// Returns a HTTP 200 for stale requests, otherwise an HTTP 302 (Found) for
    /// requests where If-None-Match headers weakly match the ETag
    pub fn into_response<T>(self, req: &HttpRequest<T>) -> HttpResponseBuilder {
	let is_stale = self.is_stale(req);
	let mut builder = if is_stale {
	    HttpResponse::Ok()
	} else {
	    HttpResponse::Found()
	};

	self.set_headers(&mut builder);
	builder
    }

    pub fn set_headers(&self, builder: &mut HttpResponseBuilder) {
	builder.set(self.cache_control().clone());
	if let Some(etag) = self.etag() {
	    builder.set(etag.clone());
	}
    }

    pub fn is_stale<T>(&self, req: &HttpRequest<T>) -> bool {
	let if_none_match: Result<header::IfNoneMatch, _> = header::Header::parse(req);
	let if_none_match = if_none_match.ok();

	let etag = self.etag();
	let etag = match etag {
	    Some(e) => e,
	    None => return true,
	};

	match if_none_match {
	    Some(h) => match h {
		header::IfNoneMatch::Items(entities) => !entities.iter().any(|e| etag.weak_eq(e)),
		header::IfNoneMatch::Any => true,
	    },
	    None => true,
	}
    }
}

pub fn etag_hash(s: &str) -> String {
    sha1::digest(s)
}

pub mod sha1 {
    use ring::digest;

    pub fn digest(s: &str) -> String {
	let sha = digest::digest(&digest::SHA1, s.as_bytes());
	sha.as_ref()
	    .iter()
	    .map(|b| format!("{:02x}", b))
	    .collect::<Vec<String>>()
	    .join("")
    }

    #[test]
    fn sha1_digest() {
	let sha = digest("testme");
	assert_eq!(sha, "3abef1a14ccecd20d6ce892cbe042ae6d74946c8");
    }
}
