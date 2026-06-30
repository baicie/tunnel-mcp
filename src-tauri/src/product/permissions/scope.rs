use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PermissionKind {
    Filesystem,
    Command,
    App,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PermissionAccess {
    Read,
    Write,
    Readwrite,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PermissionScope {
    pub id: String,
    pub kind: PermissionKind,
    pub pattern: String,
    pub access: PermissionAccess,
    pub require_approval: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewPermissionScope {
    pub kind: PermissionKind,
    pub pattern: String,
    pub access: PermissionAccess,
    pub require_approval: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PermissionDecision {
    pub allowed: bool,
    pub require_approval: bool,
    pub reason: String,
}

impl PermissionAccess {
    pub fn allows(&self, requested: &PermissionAccess) -> bool {
        matches!(
            (self, requested),
            (PermissionAccess::Readwrite, _)
                | (PermissionAccess::Read, PermissionAccess::Read)
                | (PermissionAccess::Write, PermissionAccess::Write)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{PermissionAccess, PermissionKind, PermissionScope};

    #[test]
    fn readwrite_scope_covers_all_access_levels() {
        assert!(PermissionAccess::Readwrite.allows(&PermissionAccess::Read));
        assert!(PermissionAccess::Readwrite.allows(&PermissionAccess::Write));
        assert!(PermissionAccess::Readwrite.allows(&PermissionAccess::Readwrite));
    }

    #[test]
    fn read_scope_rejects_write_requests() {
        assert!(PermissionAccess::Read.allows(&PermissionAccess::Read));
        assert!(!PermissionAccess::Read.allows(&PermissionAccess::Write));
        assert!(!PermissionAccess::Read.allows(&PermissionAccess::Readwrite));
    }

    #[test]
    fn write_scope_rejects_read_requests() {
        assert!(PermissionAccess::Write.allows(&PermissionAccess::Write));
        assert!(!PermissionAccess::Write.allows(&PermissionAccess::Read));
    }

    #[test]
    fn scope_serialises_to_lowercase_enums_and_camel_case_fields() {
        let scope = PermissionScope {
            id: "1".to_string(),
            kind: PermissionKind::Filesystem,
            pattern: "~/Documents/**".to_string(),
            access: PermissionAccess::Readwrite,
            require_approval: true,
        };
        let json = serde_json::to_string(&scope).unwrap();
        assert!(json.contains("\"kind\":\"filesystem\""));
        assert!(json.contains("\"access\":\"readwrite\""));
        assert!(json.contains("\"requireApproval\":true"));
    }
}
