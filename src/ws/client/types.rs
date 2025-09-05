use futures::stream;
use tokio_tungstenite::tungstenite;

pub type WsWriteStream = stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    tungstenite::Message,
>;
pub type WsReadStream = stream::SplitStream<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
>;

pub type ConnectionError = String;

#[derive(Debug, Clone)]
pub enum RequestSendError {
    Send,
    Disconnected,
    Serialization,
}

#[derive(Debug, Clone)]
pub enum ResponseReceiveError {
    Error,
    Deserialization,
    Disconnected,
}

pub enum State {
    Connected,
    Disconnected,
}

impl From<serde_json::Error> for ResponseReceiveError {
    fn from(value: serde_json::Error) -> Self {
        tracing::error!("Websocket deserialization error {:?}", value);

        ResponseReceiveError::Deserialization
    }
}

impl From<serde_json::Error> for RequestSendError {
    fn from(value: serde_json::Error) -> Self {
        tracing::error!("Websocket serialization error {:?}", value);

        RequestSendError::Serialization
    }
}

impl From<tungstenite::Error> for RequestSendError {
    fn from(value: tungstenite::Error) -> Self {
        tracing::error!("Websocket send error {:?}", value);

        RequestSendError::Send
    }
}
