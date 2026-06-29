use tauri::AppHandle;

use super::brand::APP_BRAND;

pub fn setup_tray(_app: &AppHandle) -> tauri::Result<()> {
    // The shell template does not install a real tray menu — only the
    // brand-defined tooltip. If you add a real menu later, keep its
    // identity sourced from `APP_BRAND.tray_tooltip` rather than any
    // product-specific marker.
    Ok(())
}

pub fn update_tray_menu(_app: &AppHandle) -> tauri::Result<()> {
    // The shell template does not manage tray menu contents. Kept as a
    // no-op so the registered command set stays consistent with the
    // shell runtime boundary.
    Ok(())
}

pub fn tray_tooltip() -> &'static str {
    APP_BRAND.tray_tooltip
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_tooltip_should_use_brand() {
        assert_eq!(tray_tooltip(), APP_BRAND.tray_tooltip);
    }

    #[test]
    fn tray_tooltip_should_not_use_legacy_brand() {
        let tooltip = tray_tooltip().to_ascii_lowercase();
        let legacy = String::from_iter(['c', 'c']);
        assert!(!tooltip.contains(&format!("{legacy} ")));
        assert!(!tooltip.contains(&format!("{legacy}-")));
    }
}
