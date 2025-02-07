use crate::events::IntoEvent;

pub struct AccessControlRoleUpdatedEvent<'a> {
    pub role: &'a str,
    pub admin_role: &'a str,
}

impl IntoEvent for AccessControlRoleUpdatedEvent<'_> {
    fn event_name(&self) -> &str {
        "access_control_role_updated"
    }

    fn event_attributes(&self) -> Vec<(String, String)> {
        vec![
            ("role".to_string(), self.role.to_string()),
            ("admin_role".to_string(), self.admin_role.to_string()),
        ]
    }
}

pub struct AccessControlRoleRemovedEvent<'a> {
    pub role: &'a str,
}

impl IntoEvent for AccessControlRoleRemovedEvent<'_> {
    fn event_name(&self) -> &str {
        "access_control_role_removed"
    }

    fn event_attributes(&self) -> Vec<(String, String)> {
        vec![("role".to_string(), self.role.to_string())]
    }
}

pub struct AccessControlHasRoleUpdatedEvent<'a> {
    pub role: &'a str,
    pub addr: &'a str,
}

impl IntoEvent for AccessControlHasRoleUpdatedEvent<'_> {
    fn event_name(&self) -> &str {
        "access_control_has_role_updated"
    }

    fn event_attributes(&self) -> Vec<(String, String)> {
        vec![
            ("role".to_string(), self.role.to_string()),
            ("addr".to_string(), self.addr.to_string()),
        ]
    }
}

pub struct AccessControlHasRoleRemovedEvent<'a> {
    pub role: &'a str,
    pub addr: &'a str,
}

impl IntoEvent for AccessControlHasRoleRemovedEvent<'_> {
    fn event_name(&self) -> &str {
        "access_control_has_role_removed"
    }

    fn event_attributes(&self) -> Vec<(String, String)> {
        vec![
            ("role".to_string(), self.role.to_string()),
            ("addr".to_string(), self.addr.to_string()),
        ]
    }
}
