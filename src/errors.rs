use nultr_shared_lib::request::{
    AuthenticatedUnexpectedErrorResponse, GetMessagesErrorResponse, GetRoomsErrorResponse,
    GetUsersResponse, LoginErrorResponse, UnexpectedErrorResponse,
};
use rust_api_kit::http::client::RequestError;

use crate::ws;

pub trait IntoErrorMessage {
    fn into_error_message(self) -> String;
}

macro_rules! define_error_messages {
    (
        $( $type:path => {
            $( $variant:pat => $msg:expr ),* $(,)?
        } ),* $(,)?
    ) => {
        $(
            impl IntoErrorMessage for $type {
                fn into_error_message(self) -> String {
                    let message = match self {
                        $( $variant => $msg, )*
                    };
                    message.to_string()
                }
            }
        )*
    };
}

define_error_messages! {
    RequestError => {
        RequestError::Deserialize => "Deserialization error",
        RequestError::Builder => "Request builder error",
        RequestError::Http(status_code) => &format!("Http error: {status_code}"),
        RequestError::Timeout => "Request timeout",
        RequestError::Connect => "Connection error",
        RequestError::Redirect => "Redirect error",
        RequestError::Unknown => "Unknown error",
        RequestError::Decode => "Deserialization error"
    },
    ws::controller::Error => {
        ws::controller::Error::Connection => "Connection error",
        ws::controller::Error::Send => "Send error",
        ws::controller::Error::Disconnected => "Disconnected",
        ws::controller::Error::Deserialization => "Deserialization error",
        ws::controller::Error::Serialization => "Serialization error",
        ws::controller::Error::Unknown => "Unknown error",
        ws::controller::Error::WrongRequestFormat => "Wrong request format",
        ws::controller::Error::UserNotFound => "User not found",
        ws::controller::Error::MessageNotFound(_) => "Message not found",
        ws::controller::Error::NotMemberOfRoom => "User is a member of this chat",
    },
    AuthenticatedUnexpectedErrorResponse => {
        AuthenticatedUnexpectedErrorResponse::InternalServerError => "Server error",
        AuthenticatedUnexpectedErrorResponse::InvalidToken => "Invalid token",
    },
    UnexpectedErrorResponse => {
        UnexpectedErrorResponse::InternalServerError => "Server error",
    },
    LoginErrorResponse => {
        LoginErrorResponse::AccessDenied => "User not found",
    },
    GetUsersResponse => {
        _ => "Unknown error"
    },
    GetMessagesErrorResponse => {
        GetMessagesErrorResponse::RoomNotFound => "User not found",
        GetMessagesErrorResponse::AccessDenied => "Access denied",
        GetMessagesErrorResponse::NotMemberOfRoom => "User is not member of chat"
    },
    GetRoomsErrorResponse => {
        GetRoomsErrorResponse::UserNotFound => "User not found"
    }
}

#[macro_export]
macro_rules! define_error_event_enum {
    (
        $enum_name:ident {
            $( $variant:ident ( $type:ty ) ),* $(,)?
        }
    ) => {
        #[derive(Clone, Debug)]
        pub enum $enum_name {
            $( $variant($type), )*
        }

        $(
            impl From<$type> for $enum_name {
                fn from(value: $type) -> Self {
                    $enum_name::$variant(value)
                }
            }
        )*
    };
}
