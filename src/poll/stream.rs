use futures::Stream;
use std::net;
use std::task::Poll;
use std::{
    collections::VecDeque,
    io::{self, Read},
    marker,
};

const CHUNK_SIZE: usize = 8192;
pub struct ReadableStream<I>
where
    I: Read,
{
    reader: io::BufReader<I>,
    buffer: VecDeque<u8>,
}

impl<I> ReadableStream<I>
where
    I: io::Read,
{
    pub fn new(stream: I) -> Self {
        ReadableStream {
            reader: io::BufReader::new(stream),
            buffer: VecDeque::with_capacity(CHUNK_SIZE),
        }
    }
    pub fn into_parts(mut self)->(I,Vec<u8>){
        todo!()
        // let buffer_leftover=self.buffer.make_contiguous();
        // let reader_leftover=self.reader.buffer();

        // (self.reader.into_inner(),reader_leftover)
        // todo!()
    }

}
impl ReadableStream<net::TcpStream> {
    pub fn from_tcp(stream: &net::TcpStream) -> Result<Self, io::Error> {
        let stream = stream.try_clone()?;
        Ok(Self::new(stream))
    }
}

impl<I> Stream for ReadableStream<I>
where
    I: Read + marker::Unpin,
{
    type Item = u8;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let s = self.get_mut();
        if !s.buffer.is_empty() {
            Poll::Ready(s.buffer.pop_front())
        } else {
            let mut buf = vec![0_u8; CHUNK_SIZE];
            let byte_read = match s.reader.read(&mut buf) {
                Ok(x) => x,
                Err(_) => {
                    return Poll::Ready(None);
                }
            };
            let mut buf = VecDeque::from(buf[0..byte_read].to_vec());
            s.buffer.append(&mut buf);
            Poll::Ready(s.buffer.pop_front())
        }
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
        let expect_result="HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<!DOCTYPE html><html><head><title>Bye-bye baby bye-bye</title><style>body { background-color: #111 }h1 { font-size:4cm; text-align: center; color: black; text-shadow: 0 0 2mm red}</style></head><body><h1>Some random content</h1></body></html>\r\n".as_bytes();
        let stream = net::TcpStream::connect("127.0.0.0:8000").unwrap();
        let mut reader = ReadableStream::new(stream);
        let content = reader.collect::<Vec<u8>>().await;
        assert_eq!(
            String::from_utf8_lossy(expect_result),
            String::from_utf8_lossy(&content)
        );
    }
}
