use crate::events::IntoEvent;
pub struct ContractPausedEvent<'a> {
    pub since_block: &'a u64,
}

impl IntoEvent for ContractPausedEvent<'_> {
    fn event_name(&self) -> &str {
        "contract_paused"
    }

    fn event_attributes(&self) -> Vec<(String, String)> {
        vec![("since_block".to_string(), self.since_block.to_string())]
    }
}

pub struct ContractResumedEvent {}

impl IntoEvent for ContractResumedEvent {
    fn event_name(&self) -> &str {
        "contract_resumed"
    }

    fn event_attributes(&self) -> Vec<(String, String)> {
        vec![]
    }
}
