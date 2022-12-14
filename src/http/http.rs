use std::{
    borrow::Cow,
    io::{self},
    marker::PhantomData,
};

use crate::poll::block::*;

use super::{header, startline};

pub mod stage {
    pub struct StartLine;
    pub struct HeaderField;
    pub struct MessageBody;
}

fn trim_ending(input: Cow<[u8]>) -> Cow<[u8]> {
    Cow::from(input.as_ref()[0..input.len() - 2].to_vec())
}

/// Implementation of http standard (stateless)
pub struct Model<C, S>
where
    C: io::Write + io::Read + std::marker::Unpin,
{
    block: Block<C>,
    stage: PhantomData<S>,
}

impl<C, S> Model<C, S>
where
    C: io::Write + io::Read + std::marker::Unpin,
{
    pub fn new(stream: C) -> Model<C, S> {
        let block = Block::new(stream);
        Model {
            block,
            stage: PhantomData,
        }
    }
    pub fn into_parts(self) -> (C, Vec<u8>, Vec<u8>) {
        self.block.into_parts()
    }
}

impl<C> Model<C, stage::StartLine>
where
    C: io::Write + io::Read + std::marker::Unpin,
{
    pub async fn next(&mut self) -> Result<Option<startline::StartLine>, startline::Error> {
        if 0 == self.block.buffer_size() {
            let buf = self.block.next_line().await;
            let start_line = trim_ending(buf);
            let start_line = start_line.try_into()?;
            Ok(Some(start_line))
        } else {
            Ok(None)
        }
    }
    pub fn skip(self) -> Model<C, stage::HeaderField> {
        #[cfg(debug_assertions)]
        assert_ne!(self.block.buffer_size(), 0);
        Model {
            block: self.block,
            stage: PhantomData,
        }
    }
}

impl<C> Model<C, stage::HeaderField>
where
    C: io::Write + io::Read + std::marker::Unpin,
{
    pub async fn next(&mut self) -> Result<Option<header::Header>, header::Error> {
        let buf = self.block.next_line().await;
        if buf.len() <= 2 {
            Ok(None)
        } else {
            let header = trim_ending(buf).to_owned().try_into()?;
            Ok(Some(header))
        }
    }
    pub fn skip(self) -> Model<C, stage::MessageBody> {
        Model {
            block: self.block,
            stage: PhantomData,
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use super::*;
    use crate::config::prelude::hash;

    #[object::test]
    async fn startline_parsing() {
        let stream = fs::File::open("test/startline").unwrap();
        let mut model = Model::<fs::File, stage::StartLine>::new(stream);

        let result1 = model.next().await.unwrap().unwrap();
        assert_eq!(
            result1,
            startline::StartLine {
                method: startline::Method::GET,
                version: startline::HttpVersion::HTTP1,
                path: b"http://a.example.com/index.html".to_vec()
            }
        );

        let result2 = model.next().await.unwrap();
        assert_eq!(result2, None);
    }

    #[object::test]
    async fn headerfield_parsing() {
        let stream = fs::File::open("test/headerfield").unwrap();
        let mut model = Model::<fs::File, stage::HeaderField>::new(stream);

        let result1 = model.next().await.unwrap().unwrap();
        assert_eq!(result1, header::Header::Host(hash(b"a.example.com")));

        let result2 = model.next().await.unwrap();
        assert_eq!(result2, None);
    }
}
