mod config;
mod http;
mod poll;
mod pool;

use config::prelude::*;
use http::prelude::*;
use pool::*;
use std::net;
use std::sync::Arc;

fn main() {
    let config = Arc::new(AppState::new("config.yml"));

    let thread = config.thread;
    let addr = config.addr.clone();

    let listener = net::TcpListener::bind(addr.clone()).unwrap();

    let mut pool = Pool::new((thread).try_into().unwrap(), &future_handler);

    println!(
        "running on system with {:?} threads on address {:?}",
        thread, addr
    );

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(handle_request((config.clone(), stream)));
    }
}

fn future_handler(future: impl futures::Future<Output = ()>) {
    futures::executor::block_on(future);
}

async fn handle_request(config: (Arc<AppState>, net::TcpStream)) {
    macro_rules! log_err {
        ($i:expr) => {
            match $i {
                Ok(x) => x,
                Err(x) => {
                    match x {
                        Error::ClientIncompatible => println!("Bad request from downstream"),
                        Error::ServerIncompatible => println!("Bad request from upstream"),
                        Error::BadProtocal => println!("Protocal not supported"),
                    }
                    return;
                }
            }
        };
    }

    let (state, client_stream) = config;
    let request = log_err!(Request::new(&client_stream));

    let request = log_err!(request.parse().await);

    let server_stream = log_err!(request.send(state.as_ref()).await);

    log_err!(reverse_proxy(server_stream, client_stream).await);
}
