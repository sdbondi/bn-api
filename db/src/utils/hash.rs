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
