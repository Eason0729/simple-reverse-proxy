use super::stream::ReadableStream;
use futures::StreamExt;
use std::{borrow::Cow, io, net, ops};

const BUFFER_SIZE: usize = 8192;
const READ_UNTIL_LIMIT: usize = 512;

pub struct Block<T>
where
    T: io::Read + std::marker::Unpin,
{
    buffer: Vec<u8>,
    reader: ReadableStream<T>,
}

impl Block<io::BufReader<net::TcpStream>> {
    pub fn from_buf_tcp(
        stream: &net::TcpStream,
    ) -> Result<Block<io::BufReader<net::TcpStream>>, io::Error> {
        let stream = stream.try_clone()?;
        let reader = io::BufReader::new(stream);
        Ok(Self::new(reader))
    }
}

impl<T> Block<T>
where
    T: io::Read + std::marker::Unpin,
{
    pub fn new(stream: T) -> Block<T> {
        Block {
            buffer: Vec::with_capacity(BUFFER_SIZE),
            reader: ReadableStream::new(stream),
        }
    }
    pub async fn next_line(&mut self) -> Cow<[u8]> {
        self.read_until(&[13, 10]).await
    }
    pub async fn read_until(&mut self, split: &[u8]) -> Cow<[u8]> {
        let start = self.buffer.len();
        let mut end = self.buffer.len();
        let mut split_iter = 0;
        while let Some(current) = self.reader.next().await {
            if self.buffer.len() >= READ_UNTIL_LIMIT {
                break;
            }
            end += 1;
            self.buffer.push(current);

            if split[split_iter] == current {
                split_iter += 1;
            } else {
                split_iter = 0;
            }

            if split_iter == split.len() {
                return Cow::from(Cow::from(&self.buffer[start..end]));
            }
        }
        Cow::from(Cow::from(&self.buffer[start..end]))
    }
    // pub fn inner_buffer(&mut self) -> Cow<[u8]> {
    //     Cow::from(self.buffer.as_slice())
    // }
    pub fn buffer_size(&self) -> usize {
        self.buffer.len()
    }
    // pub fn inner(self) -> ReadableStream<T> {
    //     self.reader
    // }
    pub fn into_parts(self) -> (T, Vec<u8>, Vec<u8>) {
        let (reader, unread_buffer) = self.reader.into_parts();
        let read_buffer = self.buffer;

        (reader, read_buffer, unread_buffer)
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use super::*;

    #[async_std::test]
    async fn read_until() {
        let file = fs::File::open("test/res2").unwrap();

        let mut block = Block::new(file);

        let line1 = block.read_until(b"\r\n").await.into_owned();
        let line2 = block.read_until(b"\r\n").await.into_owned();
        assert_eq!(line1, b"HTTP/1.1 400 Bad Request\r\n");
        assert_eq!(line2, b"Server: nginx\r\n");
    }
}
