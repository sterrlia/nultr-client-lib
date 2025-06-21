mod types;
pub use types::*;

use async_stream::stream;
use futures::{SinkExt, Stream, StreamExt as FuturesStreamExt, stream};
use shared_lib::request::{
    AuthToken, WsErrorResponse, WsMessageRequest, WsMessageResponse, WsOkResponse, WsRequest,
};
use tokio::{
    select,
    sync::mpsc::{self},
};

use super::client::{self, ResponseReceiveError};

pub fn iced_subscription() -> impl Stream<Item = Result<Event, Error>> {
    stream! {
        let (send_tx, send_rx) = mpsc::unbounded_channel::<SendEvent>();

        let mut handler = EventHandler::new(send_rx);

        yield Ok(Event::Ready(send_tx));

        loop {
            let event = handler.next().await;

            let event_result = match event {
                ReceivedEventVariant::Send(result) => {
                    handler.handle_send(result).await
                },
                ReceivedEventVariant::Receive(result) => {
                    handler.handle_receive(result).await
                }
            };

            yield event_result;
        }
    }
}

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

    pub async fn handle_send(&mut self, event: SendEvent) -> Result<Event, Error> {
        match event {
            SendEvent::Connect { url, token } => {
                let result = self.ws_client.connect(url, token).await;

                match result {
                    Err(error) => Err(error.into()),
                    Ok(_) => Ok(Event::Connected),
                }
            }
            SendEvent::Disconnect => {
                self.ws_client.disconnect().await;

                Ok(Event::Disconnected)
            }
            SendEvent::Message(request) => {
                let request = WsRequest::Message(request);
                let result = self.ws_client.send(request).await;

                match result {
                    Err(error) => Err(error.into()),
                    Ok(_) => Ok(Event::MessageSent),
                }
            }
        }
    }

    pub async fn handle_receive(
        &mut self,
        event_result: Result<Result<WsOkResponse, WsErrorResponse>, ResponseReceiveError>,
    ) -> Result<Event, Error> {
        event_result?
            .map(|ok| match ok {
                WsOkResponse::Message(ws_message_response) => Event::Message(ws_message_response),
                WsOkResponse::MessageSent => Event::MessageSent,
            })
            .map_err(|err| match err {
                WsErrorResponse::WrongFormat => Error::WrongRequestFormat,
                WsErrorResponse::WrongJsonFormat => Error::WrongRequestFormat,
                WsErrorResponse::UserNotFound => Error::UserNotFound,
                WsErrorResponse::Fatal => Error::Unknown,
            })
    }
}
