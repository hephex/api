#[cfg(feature = "use-hyper")]
extern crate hyper;

/// *api* is a library that abstracts a HTTP API
/// and separates the client from the API definition.
/// This allows you to change the underlying HTTP
/// client easily.
use std::io;
use std::collections::BTreeMap;


/// Type for the request/response headers.
pub type Headers = BTreeMap<String, Vec<String>>;
/// Type for the URL query.
pub type Query<'s> = Vec<(String, String)>;

/// Enum with all the standard HTTP methods. It also has
/// a variant `Custom` to support non-standard methods.
pub enum Method {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Patch,
    Options,
    Trace,
    Connect,
    Custom(String),
}

impl ToString for Method {
    /// Returns a string representing the HTTP method.
    fn to_string(&self) -> String {
        match *self {
            Method::Get => "GET".to_string(),
            Method::Head => "HEAD".to_string(),
            Method::Post => "POST".to_string(),
            Method::Put => "PUT".to_string(),
            Method::Delete => "DELETE".to_string(),
            Method::Patch => "PATCH".to_string(),
            Method::Options => "OPTIONS".to_string(),
            Method::Trace => "TRACE".to_string(),
            Method::Connect => "CONNECT".to_string(),
            Method::Custom(ref s) => s.clone(),
        }
    }
}

#[cfg(feature = "use-hyper")]
impl From<hyper::method::Method> for Method {
    fn from(m: hyper::method::Method) -> Method {
        match m {
            hyper::method::Method::Get => Method::Get,
            hyper::method::Method::Head => Method::Head,
            hyper::method::Method::Post => Method::Post,
            hyper::method::Method::Put => Method::Put,
            hyper::method::Method::Delete => Method::Delete,
            hyper::method::Method::Patch => Method::Patch,
            hyper::method::Method::Options => Method::Options,
            hyper::method::Method::Trace => Method::Trace,
            hyper::method::Method::Connect => Method::Connect,
            hyper::method::Method::Extension(ref s) => Method::Custom(s.clone()),
        }
    }
}

#[cfg(feature = "use-hyper")]
impl Into<hyper::method::Method> for Method {
    fn into(self) -> hyper::method::Method {
        match self {
            Method::Get => hyper::method::Method::Get,
            Method::Head => hyper::method::Method::Head,
            Method::Post => hyper::method::Method::Post,
            Method::Put => hyper::method::Method::Put,
            Method::Delete => hyper::method::Method::Delete,
            Method::Patch => hyper::method::Method::Patch,
            Method::Options => hyper::method::Method::Options,
            Method::Trace => hyper::method::Method::Trace,
            Method::Connect => hyper::method::Method::Connect,
            Method::Custom(s) => hyper::method::Method::Extension(s),
        }
    }
}


/// It represents the Server response received
/// by the client after sending a HTTP request.
pub trait HttpResponse {
    type Body: io::Read;

    /// Response's status code. It should be a integer
    /// between 100 and 599.
    fn status(&self) -> u16;

    /// Reason-phrase that describes the status code.
    /// i.e. 200 OK, 404 Not Found
    fn reason(&self) -> &str;

    /// Response's header. It contains metadata for the response.
    /// e.g. `Content-Type` informs the client about the body MIME
    /// and how to decode it.
    fn headers(&self) -> Headers;

    /// Response's body contain the data sent back from the server.
    fn body(&mut self) -> &mut Self::Body;

    /// Return `true` if the status code is 1xx, otherwise return `false`.
    fn is_1xx(&self) -> bool {
        self.status() / 100 == 1
    }

    /// Return `true` if the status code is 2xx, otherwise return `false`.
    fn is_2xx(&self) -> bool {
        self.status() / 100 == 2
    }

    /// Return `true` if the status code is 3xx, otherwise return `false`.
    fn is_3xx(&self) -> bool {
        self.status() / 100 == 3
    }

    /// Return `true` if the status code is 4xx, otherwise return `false`.
    fn is_4xx(&self) -> bool {
        self.status() / 100 == 4
    }

    /// Return `true` if the status code is 5xx, otherwise return `false`.
    fn is_5xx(&self) -> bool {
        self.status() / 100 == 5
    }
}


#[cfg(feature = "use-hyper")]
impl HttpResponse for hyper::client::Response {
    type Body = hyper::client::Response;

    fn status(&self) -> u16 {
        self.status.to_u16()
    }

    fn reason(&self) -> &str {
        self.status_raw().1.as_ref()
    }

    fn headers(&self) -> Headers {
        Headers::new()
    }

    fn body(&mut self) -> &mut hyper::client::Response {
        return self
    }
}

/// `Api` represents a HTTP API exposing all the request parameters
/// and a function to parse the HTTP response.
pub trait Api {
    type Reply;
    type Body: io::Read;
    type Error;

    /// Return the HTTP method used by this API.
    fn method(&self) -> Method;

    /// Return the URL path for this API request.
    fn path(&self) -> String;

    /// Return the URL query for this API request.
    fn query(&self) -> Query;

    /// Return the headers for this HTTP request.
    fn headers(&self) -> Headers;

    /// Return the body of this HTTP request. If the request
    /// doesn't expect any body (i.e. GET), it should return
    /// `std::io::Empty`.
    fn body(&self) -> Self::Body;

    /// Parse the HTTP response, received from the actual client,
    /// into the type `Reply`.
    fn parse<Resp>(&self, &mut Resp) -> Result<Self::Reply, Self::Error> where Resp: HttpResponse;
}


#[derive(Debug)]
pub enum SendError<S, A> {
    Client(S),
    Api(A)
}

pub trait Client<A: Api, E> {
    fn send(&mut self, url: &str, req: A) -> Result<A::Reply, SendError<E, A::Error>>;
}


#[cfg(feature = "use-hyper")]
impl<A: Api> Client<A, hyper::Error> for hyper::Client {
    /// Send an HTTP request for the given API using an `hyper` client.
    /// The path will be added do `url` that is supposed to be the base URL
    /// for the API.
    fn send(&mut self, url: &str, req: A)
        -> Result<A::Reply, SendError<hyper::Error, A::Error>>
    {
        let mut url = hyper::Url::parse(url)
            .map_err(|e| SendError::Client(hyper::Error::Uri(e)))?
            .join(req.path().as_ref())
            .map_err(|e| SendError::Client(hyper::Error::Uri(e)))?;
        let mut body = req.body();
        let body = hyper::client::Body::ChunkedBody(&mut body);

        {
            let mut query = url.query_pairs_mut();
            for (name, value) in req.query() {
                query.append_pair(name.as_str(), value.as_str());
            }
        }

        let mut headers = hyper::header::Headers::new();
        for (name, value) in req.headers() {
            headers.set_raw(
                name,
                value.iter().map(|v| v.clone().into_bytes()).collect()
            );
        }

        let mut resp = self.request(req.method().into(), url)
            .headers(headers)
            .body(body)
            .send()
            .map_err(|e| SendError::Client(e))?;

        req.parse(&mut resp)
            .map_err(|e| SendError::Api(e))
    }
}

