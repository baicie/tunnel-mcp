#[cfg(test)]
mod tests {
    use crate::product::tunnel::client_binary::{
        select_asset, TunnelClientAsset, TunnelClientManifest,
    };

    #[test]
    fn select_asset_returns_none_when_platform_not_found() {
        let manifest = TunnelClientManifest {
            version: "0.1.0".to_string(),
            assets: vec![],
        };
        assert!(select_asset(&manifest).is_none());
    }

    #[test]
    fn manifest_deserializes() {
        let raw = r#"{
          "version":"0.1.0",
          "assets":[{"platform":"darwin","arch":"arm64","url":"https://example.com/tunnel-client","sha256":"abc"}]
        }"#;
        let manifest: TunnelClientManifest = serde_json::from_str(raw).unwrap();
        assert_eq!(manifest.version, "0.1.0");
        assert_eq!(manifest.assets[0].platform, "darwin");
        let asset: TunnelClientAsset = manifest.assets[0].clone();
        assert_eq!(asset.url, "https://example.com/tunnel-client");
    }
}
