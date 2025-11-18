use game_of_estimates::adapters::sqlx::SqlxModule;
use game_of_estimates::game_server::{GameServer, GameServerAddr};
use game_of_estimates::ports::{DatabaseMigratorRef, DatabaseUrl, RoomRepositoryRef};
use std::env;
use uactor::blocking::Actor;

mod web;

#[derive(Default)]
struct MainModule;

#[derive(Clone)]
pub struct ListenAddr(String);

#[chassis::module]
impl MainModule {
    pub fn provide_database_url() -> DatabaseUrl {
        DatabaseUrl(std::env::var("DATABASE_URL").expect("DATABASE_URL should be set"))
    }

    #[chassis(singleton)]
    pub fn provide_game_server(room_repo: RoomRepositoryRef) -> GameServerAddr {
        GameServer::new(room_repo).start()
    }

    // #[chassis(singleton)]
    // pub fn provide_cert_paths() -> TlsCert {
    //     #[cfg(feature = "tls")]
    //     if let Some(cert_path) = env::var_os("GOE_PEM_PATH") {
    //         if let Some(key_path) = env::var_os("GOE_KEY_PATH") {
    //             TlsCert::Pem {
    //                 cert_path: cert_path.into(),
    //                 key_path: key_path.into(),
    //             }
    //         } else {
    //             TlsCert::Unencrypted
    //         }
    //     } else {
    //         TlsCert::Unencrypted
    //     }
    //     #[cfg(not(feature = "tls"))]
    //     TlsCert::Unencrypted
    // }

    #[chassis(singleton)]
    pub fn provide_listen_addr() -> ListenAddr {
        ListenAddr(env::var("GOE_LISTEN_ADDR").unwrap_or_else(|_| "127.0.0.1:5500".to_string()))
    }

    pub fn provide_main(game_server: GameServerAddr, listen_addr: ListenAddr) -> Main {
        Main {
            game_server,
            listen_addr,
        }
    }
}

#[chassis::injector(modules = [MainModule, SqlxModule])]
pub trait Integrator {
    fn provide_main(&self) -> Main;
    fn provide_db_migrator(&self) -> DatabaseMigratorRef;
}

pub struct Main {
    game_server: GameServerAddr,
    listen_addr: ListenAddr,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Unable to load .env files");

    let integrator = <dyn Integrator>::new().expect("Unable to build injector");

    integrator
        .provide_db_migrator()
        .migrate()
        .await
        .expect("Unable to migrate database");

    let main = integrator.provide_main();
    eprintln!("Listening on http://{}", main.listen_addr.0);
    web::main(main.game_server, main.listen_addr).await
}
