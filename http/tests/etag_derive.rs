#[cfg(test)]
mod test {

    #[test]
    fn struct_test() {
	#[derive(Serialize, ToETag)]
	#[etag_field = "a"]
	struct T {
	    a: u32,
	    b: String,
	}

	let t = T { a: 123, b: "123".to_string()};
	let etag = t.to_etag();

	assert_eq!(format!("{}", etag), "123");
    }
}
