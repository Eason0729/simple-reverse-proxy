use futures::StreamExt;

use crate::config;
use crate::poll::network::{StreamReader, StreamWriter};

use super::header;
use super::{http::*, startline};
use std::cmp;
use std::{net, time::Duration};

const CHUNK_SIZE: usize = 1024;

// When error happen, drop the request
// print error and panic if cfg(debug_assertions) is enable
#[derive(Debug)]
pub enum Error {
    ClientIncompatible,
    ServerIncompatible,
    BadProtocal,
}

// Come with a marco
#[cfg(debug_assertions)]
macro_rules! recover {
    ($i:expr,$e:expr) => {
        $i.unwrap()
    };
}
#[cfg(not(debug_assertions))]
macro_rules! recover {
    ($i:expr,$e:expr) => {
        $i.map_err(|_| $e)?
    };
}

pub struct Request<S> {
    model: Model<net::TcpStream, S>,
    keep_alive: usize,
    content_length: usize,
    host: Vec<u8>,
}

impl Request<stage::StartLine> {
    pub fn new(stream: &net::TcpStream) -> Result<Request<stage::StartLine>, Error> {
        let model = Model::from_tcp(stream).unwrap();
        Ok(Request {
            model,
            keep_alive: 2,
            content_length: 0,
            host: Vec::new(),
        })
    }

    pub async fn parse(mut self) -> Result<Request<stage::MessageBody>, Error> {
        let startline = recover!(self.model.next().await, Error::ClientIncompatible).unwrap();
        let mut model = self.model.skip();

        while let Some(header) = recover!(model.next().await, Error::ClientIncompatible) {
            match header {
                header::Header::ContentLength(x) => self.content_length = x,
                header::Header::Host(x) => self.host = x,
                header::Header::Unknown(x) => {
                    #[cfg(debug_assertions)]
                    println!("{}", String::from_utf8_lossy(&x));
                },
                header::Header::TransferEncoding => {
                    return Err(Error::BadProtocal);
                }
                header::Header::Connection(x) => {
                    if x == header::ConnectionState::Upgrade {
                        self.keep_alive = 3600 * 24;
                    }
                }
                header::Header::KeepAlive(x) => {
                    self.keep_alive = x;
                }
            }
        }
        let request = Request {
            model: model.skip(),
            keep_alive: self.keep_alive,
            content_length: self.content_length,
            host: self.host,
        };

        Ok(request)
    }
}

impl Request<stage::MessageBody> {
    pub async fn send(
        mut self,
        config: &config::Config,
        // addr: net::SocketAddr,
    ) -> Result<StreamWriter<net::TcpStream>, Error> {
        let (block, mut reader) = self.model.into_parts();

        let reader = &mut reader;

        let addr = match config.domain_mapping.get(&self.host) {
            Some(x) => x,
            None => {
                return Err(Error::ClientIncompatible);
            }
        };

        let upstream = recover!(net::TcpStream::connect(addr), Error::ServerIncompatible);
        let mut writer = StreamWriter::new(upstream);

        recover!(writer.write(block).await, Error::ServerIncompatible);

        loop {
            if self.content_length == 0 {
                break;
            }
            let plan_to_read = cmp::min(self.content_length, CHUNK_SIZE);
            let chunk: Vec<u8> = reader.take(plan_to_read).collect().await;

            self.content_length -= plan_to_read;

            recover!(writer.write(chunk).await, Error::ServerIncompatible);
        }

        Ok(writer)
    }
}

pub async fn reverse_proxy(client: net::TcpStream, server: net::TcpStream) {
    let mut writer = StreamWriter::new(server);
    let mut reader = StreamReader::new(client);

    loop {
        let item = reader.next().await;
        match item {
            Some(x) => {
                if writer.write([x].to_vec()).await.is_err() {
                    break;
                };
            }
            None => {
                break;
            }
        };
    }
}
