use actix_web::http::header;
pub use actix_web::http::header::{ETag, EntityTag};
//::{
//    CacheControl as ActixCacheControl,
//    CacheDirective,
//    IntoHeaderValue,
//    Header,
//    HeaderValue,
//    ETag,
//    InvalidHeaderValueBytes,
//};
use bigneon_db::models::Payload;
use crypto::sha1::Sha1;

pub struct CacheHeaders{
    control: header::CacheControl,
    etag: Option<ETag>
};

pub struct CacheInfo {
    max_age: u32,
    public: bool,
//    no_cache: bool,

}

pub trait ToETag {
    fn to_etag(&self) -> header:ETag;
}

struct PayloadETag<T> {
    payload: Payload<T>,
}

impl PayloadETag<T> {
    fn wrap(payload: Payload<T>) -> Self {
	Self {payload}
    }
}

impl<T> ToETag for PayloadETag<T>
    where T: ToETag {
    fn to_etag(&self) -> ETag {
	// Hash each payload entry
	let sha_strs = self.payload.data.iter()
	    .map(|d| d.to_etag())
	    .map(|etag| acc + {
		let mut sha = Sha1::new();
		sha.input_str(&format!("{}", etag));
		sha.result_str()
	    })
	    .collect::<Vec<String>>()
	    .join("");

	// Hash the resultant string
	let sha = Sha::new();
	sha.input_str(sha_strs);
	ETag(EntityTag::weak(sha))
    }
}


impl CacheHeaders {
    pub fn from_payload<T>(payload: Payload<T>) -> Self {
	CacheHeaders::from(PayloadETag::wrap(payload), opts)
    }

    pub fn from(payload: IntoETag, opts: CacheOptions) -> Self {
	let mut directives = vec![
	    header::CacheDirective::MaxAge(info.max_age),
	];

	if !info.public {
	    directives.push(header::CacheDirective::Private);
	}

	let etag = header::ETag::strong(format!("abcd{}", payload.paging.total));

	CacheHeaders(header::CacheControl(directives), etag)
    }

    pub fn inner(self) -> header::CacheControl {
	self.0
    }

    pub fn from_timestamp() -> Self {
	unimplemented!();
    }
}

impl IntoHeaderValue for CacheHeaders {
    type Error = header::InvalidHeaderValueBytes;

    fn try_into(self) -> Result<header::HeaderValue, Self::Error> {
	self.inner.try_into()
//        let mut writer = Writer::new();
//        let _ = write!(&mut writer, "{}", self);
//        header::HeaderValue::from_shared(writer.take())
    }
}
