#[cfg(test)]
mod tests {
    use crate::product::tunnel::client_download::verify_sha256;

    #[test]
    fn verify_sha256_accepts_matching_hash() {
        let hash = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
        assert!(verify_sha256(b"hello", hash).is_ok());
    }

    #[test]
    fn verify_sha256_rejects_mismatch() {
        assert!(verify_sha256(b"hello", "bad").is_err());
    }
}
