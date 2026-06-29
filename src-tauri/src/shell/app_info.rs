use serde::Serialize;

use super::brand::APP_BRAND;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub identifier: String,
    pub description: String,
}

pub fn app_info() -> AppInfo {
    AppInfo {
        name: APP_BRAND.app_name.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        identifier: APP_BRAND.identifier.to_string(),
        description: APP_BRAND.description.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_info_should_use_brand() {
        let info = app_info();

        assert_eq!(info.name, APP_BRAND.app_name);
        assert_eq!(info.identifier, APP_BRAND.identifier);
        assert_eq!(info.description, APP_BRAND.description);
        assert!(!info.version.trim().is_empty());
    }
}
