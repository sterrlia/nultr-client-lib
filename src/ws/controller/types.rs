use nultr_shared_lib::request::{
    AuthToken, WsErrorResponse, WsMarkMessagesReadRequest, WsMessageRequest, WsOkResponse
};
use url::Url;
use uuid::Uuid;

use crate::ws::client;

pub enum ReceivedEventVariant {
    Send(SendEvent),
    Receive(Result<Result<WsOkResponse, WsErrorResponse>, client::ResponseReceiveError>),
}

#[derive(Debug, Clone)]
pub enum Error {
    Send,
    Connection,
    Disconnected,
    Deserialization,
    Serialization,
    WrongRequestFormat,
    UserNotFound,
    NotMemberOfRoom,
    MessageNotFound(Uuid),
    Unknown,
}

#[derive(Debug, Clone)]
pub enum SendEvent {
    Connect { url: Url, token: AuthToken },
    Disconnect,
    Message(WsMessageRequest),
    MessagesRead(WsMarkMessagesReadRequest)
}

impl From<client::ResponseReceiveError> for Error {
    fn from(value: client::ResponseReceiveError) -> Self {
        match value {
            client::ResponseReceiveError::Error => Error::Unknown,
            client::ResponseReceiveError::Deserialization => Error::Deserialization,
            client::ResponseReceiveError::Disconnected => Error::Disconnected
        }
    }
}

impl From<client::RequestSendError> for Error {
    fn from(value: client::RequestSendError) -> Self {
        match value {
            client::RequestSendError::Send => Error::Send,
            client::RequestSendError::Disconnected => Error::Disconnected,
            client::RequestSendError::Serialization => Error::Serialization,
        }
    }
}

impl From<client::ConnectionError> for Error {
    fn from(value: client::ConnectionError) -> Self {
        tracing::error!("Connection error: {:?}", value);

        Error::Connection
    }
}
