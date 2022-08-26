mod config;
mod http;
mod poll;
mod pool;

use config::*;
use pool::*;
use http::prelude::*;
use std::env::var;
use std::net;
use std::sync::Arc;
// use crate::poll::prelude as poll;

const TIMEOUT: u64 = 7200;
const KEEPALIVE_TIMEOUT: usize = 2;

struct State {
    config: Config,
}

fn main() {
    let server_socket_addr = var("SR_ADDR").unwrap_or("0.0.0.0:80".to_string());
    let listener = net::TcpListener::bind(server_socket_addr).unwrap();

    let cpus;
    unsafe {
        cpus = libc::sysconf(libc::_SC_NPROCESSORS_ONLN);
    }
    let cpus: usize = (cpus).try_into().unwrap();
    let process_thread: usize = var("P_THREAD").unwrap_or(cpus.to_string()).parse().unwrap();
    println!(
        "running on system with {:?} logic core ({:?} threads)",
        cpus, process_thread
    );

    let mut pool = Pool::new((process_thread).try_into().unwrap(), &handle_request);

    let config = Arc::new(config::Config::new());

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute((config, stream));
    }
}

async fn handle_request(config: (Arc<config::Config>, net::TcpStream)) {
    let (state,stream)=config;
    let request=match Request::new(&stream){
        Ok(x) => x,
        Err(err) => {
            println!("Unable to connect to upstream server");
            println!("{:?}",err);
            return ;
        },
    };

    todo!();
    // let request=match request.parse().await{
    //     Ok(x) => x,
    //     Err(err) => {

            
    //         return ;
    //     },
    // };
}
