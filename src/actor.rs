//! simple actor framework

use tokio::sync::mpsc;

/// address of actor
pub type Addr<T> = mpsc::Sender<T>;

/// actor
#[async_trait::async_trait]
pub trait Actor: Sync + Send {
    /// message type
    type Message: Sync + Send;

    /// get actor address
    fn addr(&self) -> Addr<Self::Message>;

    async fn recv(&mut self) -> Option<Self::Message>;

    /// run actor
    async fn run(&mut self) {
        self.setup().await;
        while let Some(msg) = self.recv().await {
            self.on_message(msg).await;
        }
        self.tear_down().await;
    }

    async fn on_message(&mut self, msg: Self::Message);

    async fn setup(&mut self) {}
    async fn tear_down(&mut self) {}
}

pub async fn run_actor<T: Actor>(mut actor: T) {
    actor.run().await
}
