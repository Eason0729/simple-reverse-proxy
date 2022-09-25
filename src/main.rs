mod config;
mod http;
// mod object;
mod poll;
mod pool;
mod state;

use http::prelude::*;
use pool::*;
use std::env::var;
use std::net;
use std::sync::Arc;

fn main() {
    let server_socket_addr = var("ADDR").unwrap_or("0.0.0.0:80".to_string());
    let listener = net::TcpListener::bind(server_socket_addr).unwrap();

    let cpus;
    unsafe {
        cpus = libc::sysconf(libc::_SC_NPROCESSORS_ONLN);
    }
    let cpus: usize = (cpus).try_into().unwrap();
    let process_thread: usize = var("THREAD").unwrap_or(cpus.to_string()).parse().unwrap();
    println!(
        "running on system with {:?} logic core ({:?} threads)",
        cpus, process_thread
    );

    let config = Arc::new(config::Config::new());
    let mut pool = Pool::new((process_thread).try_into().unwrap(), &future_handler);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(handle_request((config.clone(), stream)));
    }
}

fn future_handler(future: impl futures::Future<Output = ()>) {
    futures::executor::block_on(future);
}

async fn handle_request(config: (Arc<config::Config>, net::TcpStream)) {
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
