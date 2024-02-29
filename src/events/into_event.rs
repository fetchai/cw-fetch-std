use cosmwasm_std::{Attribute, Event};

pub trait IntoEvent {
    fn event_name(&self) -> &str;

    fn event_attributes(&self) -> Vec<(String, String)>;

    fn into_event(self) -> Event
    where
        Self: Sized,
    {
        let res_attributes: Vec<Attribute> = self
            .event_attributes()
            .into_iter()
            .map(|(key, value)| Attribute { key, value })
            .collect();

        Event::new(self.event_name()).add_attributes(res_attributes)
    }
}
