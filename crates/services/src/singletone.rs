use std::marker::PhantomData;
use std::sync::Arc;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use zbus::{interface, proxy};

use crate::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct GenericMessage<T>(pub T);

pub trait SingletoneListener<Message>: Send + Sync {
    fn on_message(&mut self, msg: Message);
}

impl<Message, L> SingletoneListener<Message> for Arc<Mutex<L>>
where
    L: SingletoneListener<Message>,
{
    fn on_message(&mut self, msg: Message) {
        self.lock().on_message(msg);
    }
}

pub struct SingletoneServer<Listener, Message>(pub Listener, pub PhantomData<Message>)
where
    Message: Serialize + Deserialize<'static> + Send + Sync + 'static,
    Listener: SingletoneListener<Message> + Send + Sync + 'static;

#[interface(name = "rs.sergioribera.sosd")]
impl<Message, Listener> SingletoneServer<Listener, Message>
where
    Message: Serialize + Deserialize<'static> + Send + Sync + 'static,
    Listener: SingletoneListener<Message> + Send + Sync + 'static,
{
    async fn process_message(&mut self, raw_message: Vec<u8>) -> zbus::fdo::Result<()> {
        let raw_message = unsafe { core::mem::transmute::<&[u8], &'static [u8]>(&raw_message) };
        let message: GenericMessage<Message> = bincode::deserialize(raw_message).unwrap();
        self.0.on_message(message.0);
        Ok(())
    }
}

// El proxy para que los clientes envíen mensajes al servidor.
#[proxy(
    interface = "rs.sergioribera.sosd",
    default_service = "rs.sergioribera.sosd",
    default_path = "/rs/sergioribera/sosd"
)]
pub trait SingletoneClient {
    async fn process_message(&self, raw_message: Vec<u8>) -> Result<()>;
}
