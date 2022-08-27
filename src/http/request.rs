use futures::{AsyncWriteExt, StreamExt};

use super::header;
use super::{http::*, startline};
use crate::config;
use crate::poll::network::{ReadWrapper, WriteWrapper};
use futures::AsyncReadExt;
use std::collections::VecDeque;
use std::io::Read;
use std::{cmp, io, marker};
use std::{net, time::Duration};

const CHUNK_SIZE: usize = 8192;

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

pub struct Request<I, S>
where
    I: io::Read + io::Write + marker::Unpin,
{
    model: Model<I, S>,
    keep_alive: usize,
    content_length: usize,
    host: Vec<u8>,
}

impl<I> Request<I, stage::StartLine>
where
    I: io::Read + io::Write + marker::Unpin,
{
    pub fn new(stream: I) -> Result<Request<I, stage::StartLine>, Error> {
        let model = Model::new(stream);
        Ok(Request {
            model,
            keep_alive: 2,
            content_length: 0,
            host: Vec::new(),
        })
    }

    pub async fn parse(mut self) -> Result<Request<I, stage::MessageBody>, Error> {
        let startline = recover!(self.model.next().await, Error::ClientIncompatible).unwrap();
        let mut model = self.model.skip();

        while let Some(header) = recover!(model.next().await, Error::ClientIncompatible) {
            match header {
                header::Header::ContentLength(x) => self.content_length = x,
                header::Header::Host(x) => self.host = x,
                header::Header::Unknown(x) => {
                    #[cfg(debug_assertions)]
                    println!("{}", String::from_utf8_lossy(&x));
                }
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

impl<I> Request<I, stage::MessageBody>
where
    I: io::Read + io::Write + marker::Unpin,
{
    pub async fn send(
        mut self,
        config: &config::Config,
        // addr: net::SocketAddr,
    ) -> Result<net::TcpStream, Error> {
        let (reader, read_buffer, unread_buffer) = self.model.into_parts();

        let mut reader = ReadWrapper::new(reader);

        let addr = match config.domain_mapping.get(&self.host) {
            Some(x) => x,
            None => {
                return Err(Error::ClientIncompatible);
            }
        };

        let mut remaining_byte = self.content_length;
        let upstream = recover!(net::TcpStream::connect(addr), Error::ServerIncompatible);
        let mut writer = WriteWrapper::new(io::BufWriter::new(recover!(
            upstream.try_clone(),
            Error::ServerIncompatible
        )));

        recover!(writer.write(&read_buffer).await, Error::ServerIncompatible);

        let byte_sent = writer
            .write(&unread_buffer[0..remaining_byte])
            .await
            .map_err(|_| Error::ServerIncompatible)?;

        remaining_byte -= byte_sent;

        let mut chunk = [0_u8; CHUNK_SIZE];

        loop {
            if remaining_byte == 0 {
                break;
            }

            let plan_to_read = cmp::min(remaining_byte, CHUNK_SIZE);

            let byte_read: usize = recover!(
                reader.read(&mut chunk[0..plan_to_read]).await,
                Error::ServerIncompatible
            );

            self.content_length -= plan_to_read;

            recover!(
                writer.write(&chunk[0..byte_read]).await,
                Error::ServerIncompatible
            );
        }

        Ok(upstream)
    }
}

pub mod reverse_proxy {
    use std::io;

    use futures::AsyncReadExt;

    use super::*;
    pub async fn reverse_proxy(
        client: net::TcpStream,
        server: net::TcpStream,
    ) -> Result<(), Error> {
        let mut writer = WriteWrapper::new(io::BufWriter::new(server));
        let mut reader = ReadWrapper::new(io::BufReader::new(client));

        let buffer = &mut [0_u8; CHUNK_SIZE].to_vec();
        loop {
            let byte_read = match reader.read(buffer).await {
                Ok(x) => x,
                Err(err) => match err.kind() {
                    io::ErrorKind::ConnectionRefused
                    | io::ErrorKind::ConnectionReset
                    | io::ErrorKind::BrokenPipe
                    | io::ErrorKind::UnexpectedEof => {
                        return Ok(());
                    }
                    _ => unreachable!(),
                },
            };
            if byte_read == 0 {
                break;
            }
            writer
                .write(&buffer[0..byte_read].to_vec())
                .await
                .map_err(|_| Error::ClientIncompatible)?;
        }

        Ok(())
    }
}
