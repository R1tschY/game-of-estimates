//! simple actor framework

use futures_util::core_reexport::future::Future;
use tokio::sync::mpsc;
pub use tokio::task::JoinHandle;

/// address of actor
pub type Addr<T> = mpsc::Sender<T>;

/// Message box for actors
pub type MailBox<T> = mpsc::Receiver<T>;

/// actor
#[async_trait::async_trait]
pub trait Actor: Send + Sized + 'static {
    /// message type
    type Message: Send;

    async fn on_message(&mut self, msg: Self::Message, ctx: &ActorContext<Self>);

    async fn setup(&mut self, _ctx: &ActorContext<Self>) {}
    async fn tear_down(&mut self, _ctx: &ActorContext<Self>) {}

    fn start(self) -> Addr<Self::Message> {
        ActorContext::<Self>::run(self)
    }
}

/// Actor context
pub trait IActorContext<A: Actor> {
    fn addr(&self) -> Addr<A::Message>;

    fn spawn<F: Future>(f: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;
}

/// Actor context
///
/// Used to get address or spawn actors
pub struct ActorContext<A: Actor> {
    tx: Addr<A::Message>,
    rx: MailBox<A::Message>,
}

impl<A: Actor> ActorContext<A> {
    pub fn new(tx: Addr<A::Message>, rx: MailBox<A::Message>) -> Self {
        Self { tx, rx }
    }

    pub fn run(actor: A) -> Addr<A::Message> {
        let (tx, rx) = mpsc::channel(64); // TODO: config to change size
        let ctx = ActorContext::<A>::new(tx.clone(), rx);
        tokio::spawn(ctx.into_future(actor));
        tx
    }

    pub async fn into_future(mut self, mut actor: A) -> () {
        actor.setup(&self).await;
        while let Some(msg) = self.rx.recv().await {
            actor.on_message(msg, &self).await;
        }
        actor.tear_down(&self).await;
    }
}

impl<A: Actor> IActorContext<A> for ActorContext<A> {
    fn addr(&self) -> Addr<A::Message> {
        self.tx.clone()
    }

    fn spawn<F: Future>(f: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        tokio::spawn(f)
    }
}
