use crate::ws::controller::{EventHandler, ReceivedEventVariant};
use async_stream::stream;
use futures::Stream;
use nultr_shared_lib::request::{UuidIdentifier, WsMessageResponse, WsMessagesReadResponse};
use tokio::sync::mpsc;

use super::{Error, SendEvent};

#[derive(Debug, Clone)]
pub enum Event {
    Ready(mpsc::UnboundedSender<SendEvent>),
    Connected,
    Message(WsMessageResponse),
    MessageSent(UuidIdentifier),
    MessagesRead(WsMessagesReadResponse),
    MessageReceived(UuidIdentifier),
    Disconnected,
}

pub fn subscription() -> impl Stream<Item = Result<Event, Error>> {
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

            match event_result {
                Ok(event_option) => {
                    if let Some(event) = event_option {
                        yield Ok(event);
                    }
                }
                Err(error) => {
                    yield Err(error);
                }
            };
        }
    }
}
