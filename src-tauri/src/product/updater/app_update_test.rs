#[cfg(test)]
mod tests {
    use super::super::app_update::no_update;

    #[test]
    fn placeholder_reports_no_update() {
        let result = no_update("0.1.0");
        assert!(!result.available);
        assert_eq!(result.current_version, "0.1.0");
        assert!(result.latest_version.is_none());
        assert!(result.notes.is_some());
    }

    #[test]
    fn placeholder_accepts_string_or_str() {
        let from_str = no_update("0.2.0");
        let from_string = no_update(String::from("0.3.0"));
        assert_eq!(from_str.current_version, "0.2.0");
        assert_eq!(from_string.current_version, "0.3.0");
    }
}
