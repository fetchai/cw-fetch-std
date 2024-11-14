use crate::events::IntoEvent;
use cosmwasm_std::{Addr, BankMsg, Coin, CosmosMsg, Empty, Response, SubMsg};

pub struct ResponseHandler<T = Empty> {
    response: Response<T>,
}

impl<T> ResponseHandler<T> {
    pub fn add_event<E: IntoEvent>(&mut self, event: E) {
        self.response.events.push(event.into_event());
    }

    pub fn add_message(&mut self, msg: impl Into<CosmosMsg<T>>) {
        self.response.messages.push(SubMsg::new(msg));
    }

    pub fn add_submessage(&mut self, msg: SubMsg<T>) {
        self.response.messages.push(msg);
    }

    pub fn into_response(self) -> Response<T> {
        self.response
    }

    pub fn add_bank_send_msg(&mut self, to_addr: &Addr, amount: Vec<Coin>) {
        self.add_message(BankMsg::Send {
            to_address: to_addr.to_string(),
            amount,
        })
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
