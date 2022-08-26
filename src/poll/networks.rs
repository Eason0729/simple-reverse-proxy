use futures::{AsyncRead, AsyncWrite};
use std::io::{Read, Write};
use std::{io, task::Poll};
use std::{marker, net};

pub struct ReadWrapper<I>
where
    I: io::Read,
{
    reader: io::BufReader<I>,
}

impl<I> ReadWrapper<I>
where
    I: io::Read,
{
    pub fn new(stream: I) -> Self {
        ReadWrapper {
            reader: io::BufReader::new(stream),
        }
    }
}
impl ReadWrapper<net::TcpStream> {
    pub fn from_tcp(stream: &net::TcpStream) -> Result<Self, io::Error> {
        let stream = stream.try_clone()?;
        Ok(Self::new(stream))
    }
}

impl<I> AsyncRead for ReadWrapper<I>
where
    I: io::Read + marker::Unpin,
{
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<io::Result<usize>> {
        #[cfg(debug_assertions)]
        assert_ne!(buf.len(),0);
        let s = self.get_mut();
        let byte_read = s.reader.read(buf)?;
        Poll::Ready(Ok(byte_read))
    }
}

pub struct WriteWrapper<I>
where
    I: io::Write,
{
    writer: io::BufWriter<I>,
}

impl<I> WriteWrapper<I>
where
    I: io::Write,
{
    pub fn new(stream: I) -> Self {
        WriteWrapper {
            writer: io::BufWriter::new(stream),
        }
    }
}
impl WriteWrapper<net::TcpStream> {
    pub fn from_tcp(stream: &net::TcpStream) -> Result<Self, io::Error> {
        let stream = stream.try_clone()?;
        Ok(Self::new(stream))
    }
}

impl<I> AsyncWrite for WriteWrapper<I>
where
    I: io::Write + marker::Unpin,
{
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<io::Result<usize>> {
        #[cfg(debug_assertions)]
        assert_ne!(buf.len(),0);
        let s = self.get_mut();
        let byte_sent = s.writer.write(buf)?;
        Poll::Ready(Ok(byte_sent))
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        let s = self.get_mut();
        s.writer.flush()?;
        Poll::Ready(Ok(()))
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        // !
        drop(self);
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod test {
    use std::net;

    use async_std::io::ReadExt;
    use futures::StreamExt;

    use super::*;

    #[async_std::test]
    async fn tcp_write() {
        // need test server
    }

    #[async_std::test]
    async fn tcp_read() {
        // sending request to a non-standard http server, which reply "Hello World msg" instantly without sending of nothing.
        let expect_result="HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<!DOCTYPE html><html><head><title>Bye-bye baby bye-bye</title><style>body { background-color: #111 }h1 { font-size:4cm; text-align: center; color: black; text-shadow: 0 0 2mm red}</style></head><body><h1>Some random content</h1></body></html>\r\n".as_bytes();
        let stream = net::TcpStream::connect("127.0.0.0:8000").unwrap();
        let mut reader = ReadWrapper::new(stream);
        let buf=&mut Vec::new();
        reader.read_to_end(buf).await.unwrap();
        assert_eq!(expect_result, buf);
    }
}





