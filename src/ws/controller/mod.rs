mod types;

#[cfg(all(feature = "dioxus-integration", feature = "iced-integration"))]
compile_error!(
    "Features `dioxus_integration` and `iced_integration` cannot be enabled at the same time."
);

#[cfg(feature = "dioxus-integration")]
pub mod dioxus_integration;
#[cfg(feature = "dioxus-integration")]
pub use dioxus_integration::Event;

#[cfg(feature = "iced-integration")]
pub mod iced_integration;
#[cfg(feature = "iced-integration")]
pub use iced_integration::Event;

pub use types::*;

use nultr_shared_lib::request::{
    WsErrorResponse, WsOkResponse, WsRequest,
};
use tokio::{
    select,
    sync::mpsc::{self},
};

use super::client::{self, ResponseReceiveError};

struct EventHandler {
    ws_client: client::Instance,
    send_rx: mpsc::UnboundedReceiver<SendEvent>,
}

impl EventHandler {
    pub fn new(send_rx: mpsc::UnboundedReceiver<SendEvent>) -> Self {
        Self {
            ws_client: client::Instance::default(),
            send_rx,
        }
    }

    pub async fn next(&mut self) -> ReceivedEventVariant {
        match self.ws_client.state {
            client::State::Connected => select! {
                send_event_value = self.send_rx.recv() => {
                    ReceivedEventVariant::Send(send_event_value.unwrap())
                },
                receive_event_value = self.ws_client.next() => {
                    ReceivedEventVariant::Receive(receive_event_value)
                }
            },
            client::State::Disconnected => {
                let event = self.send_rx.recv().await;
                ReceivedEventVariant::Send(event.unwrap())
            }
        }
    }

    pub async fn handle_send(&mut self, event: SendEvent) -> Result<Option<Event>, Error> {
        match event {
            SendEvent::Connect { url, token } => {
                let result = self.ws_client.connect(url, token).await;

                match result {
                    Err(error) => Err(error.into()),
                    Ok(_) => Ok(Some(Event::Connected)),
                }
            }
            SendEvent::Disconnect => {
                self.ws_client.disconnect().await;

                Ok(Some(Event::Disconnected))
            }
            SendEvent::Message(request) => {
                let message_uuid = request.uuid.clone();
                let ws_request = WsRequest::Message(request);
                let result = self.ws_client.send(ws_request).await;

                match result {
                    Err(error) => Err(error.into()),
                    Ok(_) => Ok(Some(Event::MessageSent(message_uuid))),
                }
            }
            SendEvent::MessagesRead(request) => {
                let ws_request = WsRequest::MessagesRead(request);
                let result = self.ws_client.send(ws_request).await;

                match result {
                    Err(error) => Err(error.into()),
                    Ok(_) => Ok(None),
                }
            }
        }
    }

    pub async fn handle_receive(
        &mut self,
        event_result: Result<Result<WsOkResponse, WsErrorResponse>, ResponseReceiveError>,
    ) -> Result<Option<Event>, Error> {
        let event = event_result?
            .map(|ok| match ok {
                WsOkResponse::Message(ws_message_response) => Event::Message(ws_message_response),
                WsOkResponse::MessagesRead(response) => Event::MessagesRead(response),
                WsOkResponse::MessageReceived(message_uuid) => Event::MessageReceived(message_uuid),
            })
            .map_err(|err| match err {
                WsErrorResponse::WrongFormat => Error::WrongRequestFormat,
                WsErrorResponse::WrongJsonFormat => Error::WrongRequestFormat,
                WsErrorResponse::MessageNotFound(message_uuid) => {
                    Error::MessageNotFound(message_uuid)
                }
                WsErrorResponse::UserNotFound => Error::UserNotFound,
                WsErrorResponse::Fatal => Error::Unknown,
                WsErrorResponse::NotMemberOfRoom => Error::NotMemberOfRoom,
            });

        event.map(|event| Some(event))
    }
}
