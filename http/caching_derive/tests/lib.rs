#[macro_use]
extern crate bigneon_caching_derive;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate serde;

#[test]
fn to_etag_json() {
    #[derive(ToETag, Serialize)]
    struct T {
        a: u32,
        b: String,
    }

    let mut t = T {
        a: 123,
        b: "123".to_string(),
    };
    let etag = t.to_etag();

    let expected = "W/\"8c251717b621febe16f7b7b06c1ad9fec1f96218\"";
    assert_eq!(format!("{}", etag), expected);

    t.a = 124;
    let etag = t.to_etag();

    let expected = "W/\"62882a264ac7eb975f1e783ca5f20c9cc39e0038\"";
    assert_eq!(format!("{}", etag), expected);
}
