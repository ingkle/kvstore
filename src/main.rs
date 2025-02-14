use anyhow::Error;
use axum::{
    extract::{DefaultBodyLimit, Path},
    routing::{delete, get, post},
    Router,
};
use axum_macros::debug_handler;
use clap::{Args, Command};
use env_logger::Builder;
use hyper::Method;
use kvstore::error::InternalError;
use signal_hook::{consts::SIGINT, iterator::Signals};
use std::net::SocketAddr;
use std::panic;
use std::process;
use std::thread;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[derive(Debug, Args)]
pub struct SlateDBOption {
    #[arg(short = 'l', long, help = "Log filters")]
    logfilter: Option<String>,

    #[arg(short = 'n', long, help = "Listen address")]
    listen: Option<String>,
}

#[debug_handler]
async fn handle_keys_get(Path(key): Path<String>) -> Result<Vec<u8>, InternalError> {
    log::info!("keys@get={}", key);

    let value = "value";

    Ok(value.into())
}

#[debug_handler]
async fn handle_keys_post(Path(key): Path<String>, body: String) -> Result<(), InternalError> {
    log::info!("keys@post={}={}", key, body);

    Ok(())
}

#[debug_handler]
async fn handle_keys_delete(Path(key): Path<String>) -> Result<(), InternalError> {
    log::info!("keys@delete={}", key);

    Ok(())
}

fn main() -> Result<(), Error> {
    let cmd = Command::new("KVStore");
    let cmd = SlateDBOption::augment_args(cmd);
    let args = cmd.get_matches();

    if let Some(filter) = args.get_one::<String>("logfilter") {
        Builder::new().parse_filters(filter.as_str()).init();
    } else {
        env_logger::init();
    }

    let mut builder = tokio::runtime::Builder::new_multi_thread();
    if let Ok(threads) = std::env::var("TOKIO_WORKER_THREADS") {
        builder.worker_threads(threads.parse().expect("could not parse worker threads"));
    }
    if let Ok(threads) = std::env::var("TOKIO_BLOCKING_THREADS") {
        builder.max_blocking_threads(threads.parse().expect("could not parse blocking threads"));
    }
    if let Ok(size) = std::env::var("TOKIO_THREAD_STACK_SIZE") {
        builder.thread_stack_size(
            size.parse::<bytesize::ByteSize>()
                .expect("could not parse thread stack size")
                .0 as usize,
        );
    }
    let runtime = builder.enable_all().build()?;

    match runtime.block_on(async {
        thread::spawn(move || {
            let mut signals = Signals::new(&[SIGINT]).unwrap();

            for signal in signals.forever() {
                log::debug!("received signal {:?}", signal);

                std::process::exit(1)
            }
        });

        panic::set_hook(Box::new(|info| {
            log::error!("{}", info);

            process::exit(1);
        }));

        let listen: SocketAddr = match args.get_one::<String>("listen") {
            Some(listen) => listen.parse()?,
            None => "0.0.0.0:7777".parse()?,
        };
        let listener = tokio::net::TcpListener::bind(listen).await?;

        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PATCH])
            .allow_headers(tower_http::cors::Any)
            .allow_origin(tower_http::cors::Any);

        let compression = CompressionLayer::new();

        let router = Router::new()
            .route("/keys/:key", get(handle_keys_get))
            .route("/keys/:key", post(handle_keys_post))
            .route("/keys/:key", delete(handle_keys_delete))
            .layer(cors)
            .layer(compression)
            .layer(DefaultBodyLimit::disable());

        log::info!("listening on {:?}", listen);

        axum::serve(listener, router).await?;

        Ok(()) as Result<(), Error>
    }) {
        Ok(()) => {}
        Err(err) => {
            log::error!("failed to run kvstore: {:?}", err);
        }
    }

    Ok(())
}
