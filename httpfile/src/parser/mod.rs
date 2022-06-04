use std::io;

use thiserror::Error;

type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    // #[error("data store disconnected")]
    // Disconnect(#[from] io::Error),
    // #[error("the data for key `{0}` is not available")]
    // Redaction(String),
    // #[error("invalid header (expected {expected:?}, found {found:?})")]
    // InvalidHeader { expected: String, found: String },
    // #[error("unknown data store error")]
    // Unknown,
    #[error("EOF before body was parsed")]
    EOF,
    #[error("input/output error: {0}")]
    Io(#[from] io::Error),
    #[error("invalid state")]
    InvalidState,
    #[error("couldn't parse http method")]
    NoMethod,
    #[error("couldn't parse http url")]
    NoUrl,
    #[error("invalid header name")]
    InvalidHeaderName,
    #[error("invalid header value")]
    InvalidHeaderValue,

    #[cfg(feature = "reqwest")]
    #[error("invalid method")]
    InvalidMethod,
    #[cfg(feature = "reqwest")]
    #[error("invalid URL")]
    InvalidURL,
    #[cfg(feature = "reqwest")]
    #[error("{0}")]
    RequestError(#[from] reqwest::Error),

    #[error("other error: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HttpRequest {
    pub comment: String,
    pub method: String,
    pub url: String,
    pub version: String,
    pub headers: Vec<Header>,
    pub body: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Header {
    pub name: String,
    pub value: String,
}

impl Header {
    fn parse(line: String) -> Result<Header> {
        let mut parts = line.split(": ");
        let name = parts.next().ok_or(Error::InvalidHeaderName)?.to_string();
        let value = parts.next().ok_or(Error::InvalidHeaderValue)?.to_string();
        Ok(Header { name, value })
    }
}

impl HttpRequest {
    fn new() -> Self {
        HttpRequest {
            comment: String::new(),
            method: String::new(),
            url: String::new(),
            version: String::from("HTTP/1.1"),
            headers: Vec::new(),
            body: String::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.url.is_empty() || self.method.is_empty()
    }

    fn parse_method_url_version(&mut self, line: &str) -> Result<()> {
        // let line = line.trim();
        let mut parts = line.split_whitespace();
        let method = parts.next().ok_or(Error::NoMethod)?;
        let url = parts.next().ok_or(Error::NoUrl)?;
        let version = parts.next();

        self.method = method.to_string();
        self.url = url.to_string();
        if let Some(version) = version {
            self.version = version.to_string();
        }

        Ok(())
    }

    #[cfg(feature = "reqwest")]
    pub fn to_reqwest(&self, client: &reqwest::Client) -> Result<reqwest::Request> {
        let method = self.method.to_uppercase();
        let method =
            reqwest::Method::from_bytes(method.as_bytes()).map_err(|_| Error::InvalidMethod)?;
        let url = reqwest::Url::parse(&self.url).map_err(|_| Error::InvalidURL)?;

        let mut req = client.request(method, url);
        if self.body.is_empty() {
            req = req.header("content-length", "0");
        } else {
            req = req.body(self.body.clone());
        }

        for h in &self.headers {
            req = req.header(h.name.to_string(), h.value.to_string());
        }

        Ok(req.build()?)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum State {
    Url,
    Headers,
    Body,
}

pub fn parse<BR: io::BufRead>(r: BR) -> Result<Vec<HttpRequest>> {
    Parser::new(r).collect()
}

struct Parser<BR: io::BufRead> {
    r: BR,
}

impl<BR: io::BufRead> Iterator for Parser<BR> {
    type Item = Result<HttpRequest>;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.parse();
        if let Err(Error::EOF) = res {
            return None;
        }

        Some(res)
    }
}

impl<BR: io::BufRead> Parser<BR> {
    fn new(r: BR) -> Self {
        Self { r }
    }

    fn parse(&mut self) -> Result<HttpRequest> {
        let mut hc = HttpRequest::new();
        let mut state = State::Url;
        let mut line = String::new();

        match self.r.fill_buf() {
            Ok(buf) => {
                if buf.is_empty() {
                    return Err(Error::EOF);
                }
            }
            Err(err) => return Err(err.into()),
        };

        loop {
            line.clear();
            match self.r.read_line(&mut line) {
                Ok(n) => {
                    if n == 0 {
                        if hc.is_empty() {
                            return Err(Error::EOF);
                        }
                        return Ok(hc);
                    }
                }
                Err(err) => return Err(err.into()),
            };

            let line = line.trim();

            match state {
                State::Url => {
                    if line.is_empty() {
                        continue;
                    }
                    if line.starts_with("###") {
                        continue;
                    }
                    if line.starts_with('#') {
                        hc.comment.push_str(line);
                        continue;
                    }

                    if let Err(err) = hc.parse_method_url_version(line) {
                        return Err(Error::Other(err.into()));
                    }
                    state = State::Headers;
                }
                State::Headers => {
                    if line.is_empty() {
                        state = State::Body;
                        continue;
                    }

                    match Header::parse(line.to_string()) {
                        Ok(h) => hc.headers.push(h),
                        Err(err) => return Err(Error::Other(err.into())),
                    };
                }
                State::Body => {
                    if line.is_empty() {
                        continue;
                    }
                    if line.starts_with("###") {
                        return Ok(hc);
                    }

                    hc.body.push_str(line);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /*
    POST https://example.com/comments HTTP/1.1
    content-type: application/json

    {
        "name": "sample",
        "time": "Wed, 21 Oct 2015 18:27:50 GMT"
    }
    */
    #[test]
    fn parse_simplest() -> Result<()> {
        let input = r#"
            POST https://example.com/comments HTTP/1.1
            content-type: application/json

            {
                "name": "sample",
                "time": "Wed, 21 Oct 2015 18:27:50 GMT"
            }
        "#;
        let expected = HttpRequest {
            comment: String::new(),
            method: "POST".to_owned(),
            url: "https://example.com/comments".to_owned(),
            version: "HTTP/1.1".to_owned(),
            headers: vec![Header {
                name: "content-type".to_owned(),
                value: "application/json".to_owned(),
            }],
            body: r#"{"name": "sample","time": "Wed, 21 Oct 2015 18:27:50 GMT"}"#.to_owned(),
        };
        let mut parser = Parser::new(io::Cursor::new(input));

        let request = parser.next().unwrap()?;
        let next = parser.next();

        assert_eq!(request, expected);
        assert!(next.is_none());

        Ok(())
    }

    #[test]
    fn parse_multiple_2() -> Result<()> {
        let input = r#"
            ###
            # Comments
            POST https://example.com/comments HTTP/1.1
            content-type: application/json

            {
                "name": "sample",
                "time": "Wed, 21 Oct 2015 18:27:50 GMT"
            }



            ###
            # GET without body and headers
            POST https://example.com/ HTTP/2.0
        "#;
        let expect_request1 = HttpRequest {
            comment: "# Comments".to_owned(),
            method: "POST".to_owned(),
            url: "https://example.com/comments".to_owned(),
            version: "HTTP/1.1".to_owned(),
            headers: vec![Header {
                name: "content-type".to_owned(),
                value: "application/json".to_owned(),
            }],
            body: r#"{"name": "sample","time": "Wed, 21 Oct 2015 18:27:50 GMT"}"#.to_owned(),
        };
        let expect_request2 = HttpRequest {
            comment: "# GET without body and headers".to_owned(),
            method: "POST".to_owned(),
            url: "https://example.com/".to_owned(),
            version: "HTTP/2.0".to_owned(),
            headers: Vec::new(),
            body: String::new(),
        };

        let requests = parse(io::Cursor::new(input))?;

        assert_eq!(requests, vec![expect_request1, expect_request2]);

        Ok(())
    }
}
