//! simple actor framework

use std::future::Future;
use std::marker::PhantomData;

use tokio::sync::mpsc;

use crate::core::AsyncSystem;
use crate::tokio::TokioSystem;

/// address of actor
pub type Addr<T> = mpsc::UnboundedSender<T>;

/// Message box for actors
pub type MailBox<T> = mpsc::UnboundedReceiver<T>;

/// actor
pub trait Actor: Send + Sized + 'static {
    /// message type
    type Message: Send;

    type Context: ActorContext<Self>; // FUTURE: = Context<Self>;

    fn on_message(&mut self, msg: Self::Message, ctx: &Self::Context);

    fn setup(&mut self, _ctx: &Self::Context) {}
    fn tear_down(&mut self, _ctx: &Self::Context) {}

    fn start(self) -> Addr<Self::Message> {
        Self::Context::run(self)
    }
}

/// Actor context
pub trait ActorContext<A: Actor>: std::marker::Sync + std::marker::Send + 'static {
    type System: AsyncSystem;

    fn addr(&self) -> Addr<A::Message>;

    fn spawn<F: Future>(f: F) -> <Self::System as AsyncSystem>::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;

    fn run(actor: A) -> Addr<A::Message>;
}

pub type Context<A> = BasicActorContext<A, TokioSystem>;

/// Actor context
///
/// Used to get address or spawn actors
pub struct BasicActorContext<A: Actor, S: AsyncSystem> {
    tx: Addr<A::Message>,
    rx: MailBox<A::Message>,
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
            rx,
            _async_system: PhantomData,
        }
    }

    async fn into_future(mut self, mut actor: A) {
        actor.setup(&self);
        while let Some(msg) = self.rx.recv().await {
            actor.on_message(msg, &self);
        }
        actor.tear_down(&self);
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

    fn spawn<F: Future>(f: F) -> S::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        S::spawn(f)
    }

    fn run(actor: A) -> Addr<A::Message> {
        let (tx, rx) = mpsc::unbounded_channel();
        let ctx = Self::new(tx.clone(), rx);
        S::spawn(ctx.into_future(actor));
        tx
    }
}
