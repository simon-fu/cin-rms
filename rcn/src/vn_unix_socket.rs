
use std::path::Path;

use anyhow::Result;
use futures::Future;
use tokio::net::UnixDatagram;

use crate::utils::actor::{ActorHandler, ActionRes, Action, Actor, AsyncHandler};

pub struct VnUnixSocket {
    actor: Actor<Handler>,
}

impl VnUnixSocket {
    pub fn bind<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let socket = UnixDatagram::bind(path)?;
        let actor = Handler::new(socket).start("vnclient".into());
        Ok(Self {
            actor,
        })
    }

    pub async fn register(&self) -> Result<()> {
        let invoker = self.actor.invoker();
        invoker.invoke(RegisterOp).await??;
        Ok(())
    }
}

struct RegisterOp;

#[async_trait::async_trait]
impl AsyncHandler<RegisterOp> for Handler {
    type Response = Result<()>; 

    async fn handle(&mut self, _req: RegisterOp) -> Self::Response {
        Ok(())
    }
}


type UnixSockAddr = tokio::net::unix::SocketAddr;

struct Handler {
    socket: UnixDatagram,
    recv_buf: Vec<u8>,
}

impl Handler {
    pub fn new(socket: UnixDatagram) -> Self {
        Self {
            socket,
            recv_buf: vec![0; 1700],
        }
    }
}

impl Handler {
    async fn handle_recv(&mut self, result: Result<(usize, UnixSockAddr)>) -> Result<()> {
        let (_len, _from) = result?;

        Ok(())
    }
}

impl ActorHandler for Handler {
    type Next = Result<(usize, UnixSockAddr)>;

    type Msg = ();

    type Result = ();

    fn into_result(self) -> Self::Result {
        ()
    }

    fn wait_next(&mut self) -> impl Future<Output = Self::Next> + Send {
        async {
            let r = self.socket.recv_from(&mut self.recv_buf).await?;
            Ok(r)
        }
    }

    fn handle_next(&mut self, next: Self::Next) -> impl Future<Output = ActionRes> + Send {
        async {
            let _r = self.handle_recv(next).await;
            Ok(Action::None)
        }
    }

}
