use actix_web::{HttpResponse, HttpRequest, http::header, dev::HttpResponseBuilder};
pub use actix_web::http::header::{ETag, EntityTag, CacheControl, CacheDirective};

pub struct CacheHeaders(
    pub header::CacheControl,
    pub Option<ETag>,
);

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

    pub fn into_response<T>(self, req: &HttpRequest<T>) -> HttpResponseBuilder {
	let mut builder = if self.is_stale(req) {
	    HttpResponse::Ok()
	} else {
	    HttpResponse::Found()
	};

	builder.set(self.cache_control().clone());
	if let Some(etag) = self.etag() {
	    builder.set(etag.clone());
	}
	builder
    }

    pub fn is_stale<T>(&self, req: &HttpRequest<T>) -> bool {
	true
    }

//    pub fn from(payload: IntoETag, opts: CacheOptions) -> Self {
//        let mut directives = vec![
//            header::CacheDirective::MaxAge(info.max_age),
//        ];
//
//        if !info.public {
//            directives.push(header::CacheDirective::Private);
//        }
//
//        let etag = header::ETag::weak(format!("abcd{}", payload.paging.total));
//
//        CacheHeaders(header::CacheControl(directives), etag)
//    }

//    pub fn into_inner(self) -> header::CacheControl {
//        self.0
//    }
}

//impl header::IntoHeaderValue for CacheHeaders {
//    type Error = header::InvalidHeaderValueBytes;
//
//    fn try_into(self) -> Result<header::HeaderValue, Self::Error> {
//        self.inner.try_into()
////        let mut writer = Writer::new();
////        let _ = write!(&mut writer, "{}", self);
////        header::HeaderValue::from_shared(writer.take())
//    }
//}
