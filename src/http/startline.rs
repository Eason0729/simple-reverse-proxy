use std::{borrow::Cow, str, str::FromStr};

#[derive(Debug, PartialEq)]
pub enum HttpVersion {
    HTTP0,
    HTTP1, // HTTP pipeline not supported
    HTTP2,
    HTTP3,
    Unknown,
}

impl TryFrom<&[u8]> for HttpVersion {
    type Error = Error;
    fn try_from(input: &[u8]) -> Result<Self, Error> {
        // dbg!(str::from_utf8(&input.clone()).unwrap());
        match input {
            b"HTTP" => Ok(HttpVersion::Unknown),
            b"HTTP/0.9" => Ok(HttpVersion::HTTP0),
            b"HTTP/1.0" => Ok(HttpVersion::HTTP1),
            b"HTTP/1.1" => Ok(HttpVersion::HTTP1),
            b"HTTP/2" => Ok(HttpVersion::HTTP2),
            b"HTTP/3" => Ok(HttpVersion::HTTP3),
            _ => Err(Error::MisMatchedValue),
        }
    }
}

impl FromStr for HttpVersion {
    type Err = ();
    fn from_str(input: &str) -> Result<HttpVersion, Self::Err> {
        match input {
            "HTTP" => Ok(HttpVersion::Unknown),
            "HTTP/0.9" => Ok(HttpVersion::HTTP0),
            "HTTP/1.0" => Ok(HttpVersion::HTTP1),
            "HTTP/1.1" => Ok(HttpVersion::HTTP1),
            "HTTP/2" => Ok(HttpVersion::HTTP2),
            "HTTP/3" => Ok(HttpVersion::HTTP3),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Method {
    GET,
    CONNECT,
    POST,
    HEAD,
    PUT,
    DELETE,
    OPTIONS,
    TRACE,
    PATCH,
}

impl TryFrom<&[u8]> for Method {
    type Error = Error;
    fn try_from(input: &[u8]) -> Result<Self, Error> {
        // dbg!(str::from_utf8(&input.clone()).unwrap());
        match input {
            b"GET" => Ok(Method::GET),
            b"POST" => Ok(Method::POST),
            b"HEAD" => Ok(Method::HEAD),
            b"PUT" => Ok(Method::PUT),
            b"DELETE" => Ok(Method::DELETE),
            b"CONNECT" => Ok(Method::CONNECT),
            b"OPTIONS" => Ok(Method::OPTIONS),
            b"TRACE" => Ok(Method::TRACE),
            b"PATCH" => Ok(Method::PATCH),
            _ => Err(Error::MisMatchedValue),
        }
    }
}

impl FromStr for Method {
    type Err = ();
    fn from_str(input: &str) -> Result<Method, Self::Err> {
        match input {
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            "HEAD" => Ok(Method::HEAD),
            "PUT" => Ok(Method::PUT),
            "DELETE" => Ok(Method::DELETE),
            "CONNECT" => Ok(Method::CONNECT),
            "OPTIONS" => Ok(Method::OPTIONS),
            "TRACE" => Ok(Method::TRACE),
            "PATCH" => Ok(Method::PATCH),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct StartLine {
    pub method: Method,
    pub version: HttpVersion,
    pub path: Vec<u8>,
}

#[derive(Debug)]
pub enum Error {
    BadFormat,
    MisMatchedValue,
}

impl TryFrom<&[u8]> for StartLine {
    type Error = Error;
    fn try_from(input: &[u8]) -> Result<Self, Error> {
        #[cfg(debug_assertions)]
        assert_ne!(input.last().unwrap(), &b"\n"[0]);

        let mut iter = input.split(|&x| x == 32);
        let method = iter.next().ok_or(Error::BadFormat)?;
        let path = iter.next().ok_or(Error::BadFormat)?;
        let version = iter.next().ok_or(Error::BadFormat)?;
        let method = method.try_into()?;

        let version = version.try_into()?;

        Ok(StartLine {
            method,
            version,
            path: path.to_vec(),
        })
    }
}

impl<'a> TryFrom<Cow<'a, [u8]>> for StartLine {
    type Error = Error;

    fn try_from(input: Cow<'a, [u8]>) -> Result<Self, Self::Error> {
        // it were `Ok(input.clone().try_into()?)`, but core dumped
        // thread '<unknown>' has overflowed its stack
        // fatal runtime error: stack overflow
        // [1]    120000 abort (core dumped)  cargo run
        Ok(input.as_ref().try_into()?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn start_line() {
        let source = b"GET http://a.example.com/index.html HTTP/1.1";
        // let source = Cow::from(source);
        let result: StartLine = source.as_ref().try_into().unwrap();

        let expect_result = StartLine {
            method: Method::GET,
            version: HttpVersion::HTTP1,
            path: b"http://a.example.com/index.html".to_vec(),
        };

        assert_eq!(expect_result, result);
    }
}
