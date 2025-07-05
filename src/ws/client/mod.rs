mod types;
use nultr_shared_lib::request::{WsErrorResponse, WsOkResponse, WsRequest, WsResponse};
pub use types::*;

use futures::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::{self, handshake::client::generate_key};
use url::Url;

pub struct Channel {
    pub write_stream: WsWriteStream,
    pub read_stream: WsReadStream,
}

impl Channel {
    pub async fn send(&mut self, request: WsRequest) -> Result<(), RequestSendError> {
        let serialized_message = serde_json::to_string(&request)?;

        let tungstenine_message = tungstenite::Message::Text(serialized_message.into());

        self.write_stream.send(tungstenine_message).await?;

        Ok(())
    }

    pub async fn next(
        &mut self,
    ) -> Result<Result<WsOkResponse, WsErrorResponse>, ResponseReceiveError> {
        let ws_stream_value = &mut self.read_stream.next().await;
        match_ws_event(ws_stream_value)
    }
}

pub struct Instance {
    pub state: State,
    channel: Option<Channel>,
}

impl Default for Instance {
    fn default() -> Self {
        Self {
            channel: None,
            state: State::Disconnected,
        }
    }
}

impl Instance {
    pub async fn connect(&mut self, url: Url, token: String) -> Result<(), ConnectionError> {
        let request = tungstenite::http::Request::builder()
            .method("GET")
            .header("Authorization", format!("Bearer {token}"))
            .header("Upgrade", "websocket")
            .header("Connection", "upgrade")
            .header("Host", url.host_str().unwrap())
            .header("Sec-WebSocket-Key", generate_key())
            .header("Sec-Websocket-Version", "13")
            .uri(url.to_string())
            .body(())
            .map_err(|err| err.to_string())?;

        let (ws_stream, _) = tokio_tungstenite::connect_async(request)
            .await
            .map_err(|err| err.to_string())?;

        let (ws_write, ws_read) = ws_stream.split();

        self.channel = Some(Channel {
            write_stream: ws_write,
            read_stream: ws_read,
        });

        self.state = State::Connected;

        Ok(())
    }

    pub async fn send(&mut self, request: WsRequest) -> Result<(), RequestSendError> {
        if let Some(channel) = &mut self.channel {
            let result = channel.send(request).await;
            if result.is_err() {
                self.disconnect().await;
            }

            result
        } else {
            self.disconnect().await;
            Err(RequestSendError::Disconnected)
        }
    }

    pub async fn next(
        &mut self,
    ) -> Result<Result<WsOkResponse, WsErrorResponse>, ResponseReceiveError> {
        if let Some(channel) = &mut self.channel {
            let next = channel.next().await;

            match next {
                Err(ResponseReceiveError::Disconnected) => {
                    self.disconnect().await;

                    Err(ResponseReceiveError::Disconnected)
                }
                next => next,
            }
        } else {
            self.disconnect().await;

            Err(ResponseReceiveError::Disconnected)
        }
    }

    pub async fn disconnect(&mut self) {
        tracing::error!("Disconnected");
        self.state = State::Disconnected;

        if let Some(channel) = &mut self.channel {
            let close_result = channel.write_stream.close().await;

            if let Err(error) = close_result {
                tracing::warn!("Disconnect error {:?}", error)
            }

            self.channel = None;
        }
    }
}

fn match_ws_event(
    receive_event_value: &mut Option<Result<tungstenite::Message, tungstenite::Error>>,
) -> Result<Result<WsOkResponse, WsErrorResponse>, ResponseReceiveError> {
    match receive_event_value {
        Some(Ok(receive_event)) => {
            let json = receive_event.to_string();
            let ws_message: WsResponse = serde_json::from_str(json.as_str())?;

            Ok(match ws_message {
                WsResponse::Ok(ws_ok_response) => Ok(ws_ok_response),
                WsResponse::Err(ws_error_response) => Err(ws_error_response),
            })
        }
        Some(Err(error)) => {
            tracing::error!("Unknown ws error: {:?}", error);

            Err(ResponseReceiveError::Error)
        }
        None => {
            tracing::error!("Websocket channel closed");

            Err(ResponseReceiveError::Disconnected)
        }
    }
}
