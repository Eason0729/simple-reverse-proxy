use std::collections::VecDeque;
use std::net;

use futures::task::Poll;
use futures::{task, AsyncWrite};
use futures::{Future, Stream};
use std::io::{self, Read, Write};
use std::pin::Pin;

const BUFFER_SIZE: usize = 4096;
// struct

#[derive(Debug)]
pub struct StreamWriter<T>
where
    T: io::Write,
{
    writer: io::BufWriter<T>,
    buffer: VecDeque<u8>,
}

// impl <T> AsyncWrite for StreamWriter<T>{ }

impl<T> StreamWriter<T>
where
    T: io::Write + std::marker::Unpin,
{
    pub fn new(stream: T) -> StreamWriter<T> {
        StreamWriter {
            writer: io::BufWriter::new(stream),
            buffer: VecDeque::new(),
        }
    }
    pub async fn write(&mut self, content: Vec<u8>) -> Result<(), std::io::Error> {
        let mut content = VecDeque::from(content);
        self.buffer.append(&mut content);
        self.await
    }
}

impl StreamWriter<net::TcpStream> {
    pub fn from_tcp_stream(
        stream: &net::TcpStream,
    ) -> Result<StreamWriter<net::TcpStream>, io::Error> {
        let stream = stream.try_clone()?;
        Ok(StreamWriter {
            writer: io::BufWriter::new(stream),
            buffer: VecDeque::new(),
        })
    }
}

impl<T> Future for StreamWriter<T>
where
    T: io::Write + std::marker::Unpin,
{
    type Output = Result<(), io::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let s = self.get_mut();
        let data = s.buffer.make_contiguous();
        let byte_sent = match s.writer.write(data) {
            Ok(x) => x,
            Err(err) => match err.kind() {
                io::ErrorKind::ConnectionRefused
                | io::ErrorKind::ConnectionReset
                | io::ErrorKind::Interrupted => {
                    return Poll::Pending;
                }
                _ => unreachable!(),
            },
        };

        s.buffer.resize(byte_sent, 0);

        return if s.buffer.is_empty() {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        };
    }
}

#[derive(Debug)]
pub struct StreamReader<T>
where
    T: io::Read,
{
    reader: io::BufReader<T>,
    buffer: VecDeque<u8>,
}

impl<T> StreamReader<T>
where
    T: io::Read,
{
    pub fn new(stream: T) -> StreamReader<T> {
        StreamReader {
            reader: io::BufReader::new(stream),
            buffer: VecDeque::new(),
        }
    }
}

impl StreamReader<net::TcpStream> {
    pub fn from_tcp(stream: &net::TcpStream) -> Result<StreamReader<net::TcpStream>, io::Error> {
        let stream = stream.try_clone()?;
        Ok(Self::new(stream))
    }
}

impl<T> Stream for StreamReader<T>
where
    T: io::Read + std::marker::Unpin,
{
    type Item = u8;
    fn poll_next(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        let s = self.get_mut();
        let buf = &mut [0_u8; BUFFER_SIZE];

        if !s.buffer.is_empty() {
            return Poll::Ready(s.buffer.pop_front());
        }

        return match s.reader.read(buf) {
            Ok(read_byte) => {
                return if read_byte == 0 {
                    Poll::Ready(None)
                } else {
                    buf[1..read_byte].iter().for_each(move |x| {
                        s.buffer.push_back(*x);
                    });
                    Poll::Ready(Some(buf[0]))
                }
            }
            Err(err) => match err.kind() {
                io::ErrorKind::NotFound
                | io::ErrorKind::ConnectionRefused
                | io::ErrorKind::ConnectionReset
                | io::ErrorKind::BrokenPipe
                | io::ErrorKind::UnexpectedEof => Poll::Ready(None),
                _ => unreachable!(),
            },
        };
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

#[cfg(test)]
mod test {
    use std::net;

    use futures::StreamExt;

    use super::*;

    #[async_std::test]
    async fn tcp_read() {
        // sending request to a non-standard http server, which reply "Hello World msg" instantly without sending of nothing.
        let expect_result="HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<!DOCTYPE html><html><head><title>Bye-bye baby bye-bye</title><style>body { background-color: #111 }h1 { font-size:4cm; text-align: center; color: black; text-shadow: 0 0 2mm red}</style></head><body><h1>Goodbye, world!</h1></body></html>\r\n".as_bytes();
        let stream = net::TcpStream::connect("127.0.0.0:8000").unwrap();
        let mut reader = StreamReader::new(stream);
        let content = reader.collect::<Vec<u8>>().await;
        assert_eq!(expect_result, content);
    }
    #[async_std::test]
    async fn tcp_write() {
        let content = "GET http://a.example.com/index.html HTTP/1.1\r\nHost: a.example.com\r\n\r\n"
            .as_bytes();
        let stream = net::TcpStream::connect("127.0.0.0:8000").unwrap();
        let mut writer = StreamWriter::new(stream);
        let a = writer.write(content.to_vec());
        // consider implmenting a regular http server
    }
    #[test]
    fn playground() {
        dbg!("bors".as_bytes());
    }
}
