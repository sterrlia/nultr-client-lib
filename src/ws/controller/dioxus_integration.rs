use nultr_shared_lib::request::WsMessageResponse;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum Event {
    Connected,
    Message(WsMessageResponse),
    MessageSent,
    Disconnected,
}

pub struct Channel<T> {
    pub tx: mpsc::UnboundedSender<T>,
    pub rx: mpsc::UnboundedReceiver<T>,
}

impl<T> Default for Channel<T> {
    fn default() -> Self {
        let (tx, rx) = mpsc::unbounded_channel::<T>();
        Self { tx, rx }
    }
}

pub trait GetChannelTrait<T> {
    fn get_tx(&self) -> &mpsc::UnboundedSender<T>;
    fn get_rx(&mut self) -> &mut mpsc::UnboundedReceiver<T>;
}

#[macro_export]
macro_rules! define_event_channels {
    (
        $struct_name:ident (
            $( $ty:ident ),* $(,)?
        )
    ) => {
        #[allow(non_snake_case)]
        pub struct $struct_name {
            $( $ty: $crate::ws::controller::dioxus_integration::Channel<$ty>, )*
        }

        impl Default for $struct_name {
            fn default() -> Self {
                Self {
                    $(
                        $ty: $crate::ws::controller::dioxus_integration::Channel::default()
                    )*
                }
            }
        }

        $(
            impl $crate::ws::controller::dioxus_integration::GetChannelTrait<$ty> for $struct_name {
                fn get_tx(&self) -> &tokio::sync::mpsc::UnboundedSender<$ty> {
                    &self.$ty.tx
                }

                fn get_rx(&mut self) -> &mut tokio::sync::mpsc::UnboundedReceiver<$ty> {
                    &mut self.$ty.rx
                }
            }
        )*
    };
}


