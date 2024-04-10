use crate::events::IntoEvent;
use cosmwasm_std::{CosmosMsg, Response, SubMsg};

pub struct ResponseHandler {
    response: Response,
}

impl ResponseHandler {
    pub fn add_event<T: IntoEvent>(&mut self, event: T) {
        self.response.events.push(event.into_event());
    }

    pub fn add_msg(&mut self, msg: impl Into<CosmosMsg>) {
        self.response.messages.push(SubMsg::new(msg));
    }

    pub fn into_response(self) -> Response {
        self.response
    }
}

impl Default for ResponseHandler {
    fn default() -> Self {
        ResponseHandler {
            response: Response::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{attr, Event};

    struct TestEvent {
        domain: String,
        nonce: u64,
    }

    impl IntoEvent for TestEvent {
        fn event_name(&self) -> &str {
            "test"
        }

        fn event_attributes(&self) -> Vec<(String, String)> {
            vec![
                ("domain".to_string(), self.domain.to_string()),
                ("nonce".to_string(), self.nonce.to_string()),
            ]
        }
    }

    #[test]
    fn test_handler() {
        let mut res = ResponseHandler::default();
        res.add_event(TestEvent {
            domain: "domain.com".to_string(),
            nonce: 123,
        });
        let expected_event = Event::new("test")
            .add_attributes(vec![attr("domain", "domain.com"), attr("nonce", "123")]);

        let response = res.into_response();

        assert_eq!(response.events.len(), 1);

        assert_eq!(response.events[0], expected_event);
    }
}
