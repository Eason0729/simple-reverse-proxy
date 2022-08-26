use std::borrow::Cow;
use std::{cmp, mem};

#[derive(Debug, PartialEq)]
pub enum Error {
    TooLargeValue,
    MisMatchedValue,
}

#[derive(Debug, PartialEq)]
pub enum ConnectionState {
    KeepAlive,
    Close,
    Upgrade,
}

impl TryFrom<Vec<u8>> for ConnectionState {
    type Error = Error;
    fn try_from(input: Vec<u8>) -> Result<Self, Error> {
        match input.as_slice() {
            b"keep-alive" => Ok(Self::KeepAlive),
            b"close" => Ok(Self::Close),
            b"upgrade" => Ok(Self::Upgrade),
            _ => Err(Error::MisMatchedValue),
        }
    }
}

fn parse_numeric(input: Vec<u8>) -> Result<usize, Error> {
    if input.len() > mem::size_of::<usize>() {
        Err(Error::TooLargeValue)
    } else {
        let mut val: usize = 0;
        for i in input {
            val *= 256;
            val += i as usize;
        }
        Ok(val)
    }
}

#[derive(Debug, PartialEq)]
pub enum Header {
    ContentLength(usize),
    Host(Vec<u8>),
    Unknown(Vec<u8>),
    TransferEncoding,
    Connection(ConnectionState),
    KeepAlive(usize),
}

impl TryFrom<Vec<u8>> for Header {
    type Error = Error;
    fn try_from(input: Vec<u8>) -> Result<Self, Error> {
        #[cfg(debug_assertions)]
        assert_ne!(input.last().unwrap(), &b"\n"[0]);
        const SPLITER1: u8 = 58;
        #[cfg(debug_assertions)]
        assert_eq!(&SPLITER1, b":".first().unwrap());
        let mut field = (0_usize, 0_usize);
        let mut value = (0_usize, 0_usize);

        let iter = input.iter();
        for iter in iter {
            field.1 += 1;
            if (*iter) == SPLITER1 {
                field.1 -= 1;
                break;
            }
        }

        value.0 = field.1 + 2;
        value.1 = input.len();

        value.0 = cmp::min(value.0, value.1);

        let field = &input[field.0..field.1];
        let value = input[value.0..value.1].to_vec();

        Ok(match field {
            b"Transfer-Encoding" => Self::TransferEncoding,
            b"Content-Length" => Self::ContentLength(parse_numeric(value)?),
            b"Host" => Self::Host(value),
            b"Connection" => Self::Connection(value.try_into()?),
            b"Keep-Alive" => Self::KeepAlive(parse_numeric(value)?),
            _ => Self::Unknown(input),
        })
    }
}

impl<'a> TryFrom<Cow<'a, [u8]>> for Header {
    type Error = Error;

    fn try_from(input: Cow<[u8]>) -> Result<Self, Self::Error> {
        Ok(input.try_into()?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn header() {
        let source = b"Host: www.example.com".to_vec();
        // let source = Cow::from(source);
        let result: Header = source.try_into().unwrap();

        let binary_host: Vec<u8> = b"www.example.com".to_vec();

        assert_eq!(Header::Host(binary_host), result);
    }

    #[test]
    fn numeric_parsing() {
        let source: Vec<u8> = [1, 212, 8, 71].to_vec();
        let result = parse_numeric(source);
        assert_eq!(30672967, result.unwrap());

        let source: Vec<u8> = [8, 4, 1, 212, 8, 4, 1, 212, 8, 4, 1, 212, 8, 71].to_vec();
        let result = parse_numeric(source);

        assert_eq!(Error::TooLargeValue, result.unwrap_err());
    }
}
