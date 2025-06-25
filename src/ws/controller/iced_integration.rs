use async_stream::stream;
use nultr_shared_lib::request::WsMessageResponse;
use tokio::sync::mpsc;
use futures::{SinkExt, Stream, StreamExt as FuturesStreamExt};
use crate::ws::controller::{EventHandler, ReceivedEventVariant};

use super::{Error, SendEvent};

#[derive(Debug, Clone)]
pub enum Event {
    Ready(mpsc::UnboundedSender<SendEvent>),
    Connected,
    Message(WsMessageResponse),
    MessageSent,
    Disconnected,
}

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

