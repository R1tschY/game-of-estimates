use std::convert::Infallible;
use std::net::SocketAddr;
#[cfg(feature = "tls")]
use std::path::PathBuf;
use std::{env, fmt};

use log::info;
use tokio::signal::unix::{signal, SignalKind};
use warp::ws::WebSocket;
use warp::Filter;

use game_of_estimates::assets;
use game_of_estimates::assets::AssetCatalog;
use game_of_estimates::game_server::{GameServer, GameServerAddr};
use game_of_estimates::player::Player;
use game_of_estimates::remote::RemoteConnection;
use uactor::blocking::Actor;

trait ErrorContextExt<T> {
    fn in_context(self, context: &str) -> Result<T, String>;
}

impl<T, E: fmt::Debug> ErrorContextExt<T> for Result<T, E> {
    fn in_context(self, context: &str) -> Result<T, String> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(format!("{}: {:?}", context, err)),
        }
    }
}

#[chassis::integration]
mod integration {
    use super::*;

    struct Bindings;

    impl Bindings {
        #[singleton]
        pub fn provide_game_server() -> GameServerAddr {
            GameServer::default().start()
        }

        #[singleton]
        pub fn provide_cert_paths() -> TlsCert {
            #[cfg(feature = "tls")]
            if let Some(cert_path) = env::var_os("GOE_PEM_PATH") {
                if let Some(key_path) = env::var_os("GOE_KEY_PATH") {
                    TlsCert::Pem {
                        cert_path: cert_path.into(),
                        key_path: key_path.into(),
                    }
                } else {
                    TlsCert::Unencrypted
                }
            } else {
                TlsCert::Unencrypted
            }
            #[cfg(not(feature = "tls"))]
            TlsCert::Unencrypted
        }

        #[singleton]
        pub fn provide_listen_addr() -> ListenAddr {
            ListenAddr(env::var("GOE_LISTEN_ADDR").unwrap_or_else(|_| "127.0.0.1:5500".to_string()))
        }

        pub fn provide_main(
            addr: ListenAddr,
            tls_cert: TlsCert,
            game_server: GameServerAddr,
        ) -> Main {
            Main {
                addr,
                tls_cert,
                game_server,
            }
        }
    }

    pub trait Integrator {
        fn provide_main(&self) -> Main;
    }
}

pub fn data<T: Clone + Send>(value: T) -> impl Filter<Extract = (T,), Error = Infallible> + Clone {
    warp::any().map(move || value.clone())
}

#[derive(Clone)]
struct ListenAddr(String);

#[derive(Clone)]
enum TlsCert {
    Unencrypted,
    #[cfg(feature = "tls")]
    Pem {
        cert_path: PathBuf,
        key_path: PathBuf,
    },
}

pub struct Main {
    addr: ListenAddr,
    tls_cert: TlsCert,
    game_server: GameServerAddr,
}

impl Main {
    pub async fn run(&self) -> Result<(), String> {
        let addr: SocketAddr = self.addr.0.parse().expect("Invalid listen address");
        let assets = Box::leak(Box::new(AssetCatalog::new()));

        let ws = warp::path("ws")
            .and(warp::path::end())
            .and(warp::ws())
            .and(data(self.game_server.clone()))
            .map(|ws: warp::ws::Ws, game_server: GameServerAddr| {
                ws.on_upgrade(move |websocket| Main::run_player(websocket, game_server))
            });

        info!("Listening on {} ...", &self.addr.0);

        let routes = ws.or(assets::assets(assets));

        match self.tls_cert.clone() {
            TlsCert::Unencrypted => warp::serve(routes).run(addr).await,
            #[cfg(feature = "tls")]
            TlsCert::Pem {
                cert_path,
                key_path,
            } => {
                warp::serve(routes)
                    .tls()
                    .cert_path(cert_path)
                    .key_path(key_path)
                    .run(addr)
                    .await;
            }
        }

        Ok(())
    }

    async fn run_player(ws: WebSocket, game_server: GameServerAddr) {
        Player::new(RemoteConnection::new(ws), game_server)
            .run()
            .await
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    env_logger::try_init().in_context("Failed to init logger")?;

    use crate::integration::Integrator;
    let integrator = integration::IntegratorImpl::new();
    let main = integrator.provide_main();

    let mut sigterm =
        signal(SignalKind::terminate()).in_context("Failed to register SIGTERM handler")?;
    let mut sigint =
        signal(SignalKind::interrupt()).in_context("Failed to register SIGINT handler")?;

    tokio::select! {
        ret = main.run() => { ret },

        _ = sigterm.recv() => {
            info!("Received SIGTERM. Shutting down ...");
            Ok(())
        },
        _ = sigint.recv() => {
            info!("Received SIGINT. Shutting down ...");
            Ok(())
        },
    }
}
