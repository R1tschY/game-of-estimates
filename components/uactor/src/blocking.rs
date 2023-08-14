//! simple actor framework

use std::future::Future;
use std::marker::PhantomData;

use tokio::sync::mpsc;

use crate::core::AsyncSystem;

/// address of actor
pub type Addr<T> = mpsc::Sender<T>;

/// Message box for actors
pub type MailBox<T> = mpsc::Receiver<T>;

/// actor
#[async_trait::async_trait]
pub trait Actor: Send + Sized + 'static {
    /// message type
    type Message: Send;

    type Context: ActorContext<Self>; // FUTURE: = Context<Self>;

    async fn on_message(&mut self, msg: Self::Message, ctx: &mut Self::Context);

    async fn setup(&mut self, _ctx: &mut Self::Context) {}
    async fn tear_down(&mut self, _ctx: &mut Self::Context) {}

    fn start(self) -> Addr<Self::Message> {
        Self::Context::run(self)
    }
}

/// Actor context
pub trait ActorContext<A: Actor>: std::marker::Sync + std::marker::Send + 'static {
    type System: AsyncSystem;

    fn addr(&self) -> Addr<A::Message>;

    /// Stop processing messages and exit actor
    fn force_quit(&mut self);

    /// Spawn coroutine
    fn spawn<F: Future>(f: F) -> <Self::System as AsyncSystem>::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;

    /// Run this actor.
    ///
    /// Note: Please use `A::start` instead.
    fn run(actor: A) -> Addr<A::Message>;
}

/// Actor context
///
/// Used to get address or spawn actors
pub struct BasicActorContext<A: Actor, S: AsyncSystem> {
    tx: Addr<A::Message>,
    rx: Option<MailBox<A::Message>>,
    _async_system: PhantomData<S>,
}

impl<A, S> BasicActorContext<A, S>
where
    A: Actor<Context = Self>,
    S: AsyncSystem,
{
    fn new(tx: Addr<A::Message>, rx: MailBox<A::Message>) -> Self {
        Self {
            tx,
            rx: Some(rx),
            _async_system: PhantomData,
        }
    }

    async fn into_future(mut self, mut actor: A) {
        actor.setup(&mut self).await;
        while let Some(rx) = self.rx.as_mut() {
            if let Some(msg) = rx.recv().await {
                actor.on_message(msg, &mut self).await;
            }
        }
        actor.tear_down(&mut self).await;
    }
}

impl<A: Actor, S: AsyncSystem> ActorContext<A> for BasicActorContext<A, S>
where
    A: Actor<Context = Self>,
    S: AsyncSystem,
{
    type System = S;

    fn addr(&self) -> Addr<A::Message> {
        self.tx.clone()
    }

    fn force_quit(&mut self) {
        self.rx = None;
    }

    fn spawn<F: Future>(f: F) -> S::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        S::spawn(f)
    }

    fn run(actor: A) -> Addr<A::Message> {
        let (tx, rx) = mpsc::channel(64); // TODO: config to change size
        let ctx = Self::new(tx.clone(), rx);
        S::spawn(ctx.into_future(actor));
        tx
    }
}
