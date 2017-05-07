# api

`api` provides a set of utilities for defining
HTTP API on the client side without worrying about
the actual HTTP client (until you send the
request!).

## Api trait

`api::Api` is a trait used to define a remote API.
The developer must implement it for the `struct`
that represents the API endpoint.

For our examples, we'll create a simple client for [httpbin][httpbin]
and its endpoint `/delay:n` (delays responding n seconds
and returns some information about the request).

`Api` has three associated types:

+ `Body` is used to generate the request body and
    it must implement the trait `std::io::Read`.
   If the endpoint doesn't require a body `std::io::Empty`
    should be used.

+ `Reply` defines the response that we expect to receive
    from the API.
+ `Error` defines all the expected errors that we could get when
    the response is received.

First of all, we need to define the request and the response.
`/delay/:n` has only one parameter in the path and we'll only care about the fields `origin` and `headers` in the JSON response
(we'll use `serde_json` to parse the response's body).

```rust
extern crate api;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

// request for /delay/:n
struct Delay {
    delay: u8
}

// it's a subset of the available data
#[derive(Deserialize)]
struct Info {
    origin: String, // ip address
    headers: BTreeMap<String, String>, // headers received by the server
}


impl api::Api for Delay {
    type Reply = Info;
    type Body = io::Empty;
    type Error = serde_json::Error;

    ...
}
```

Then, we can start defining how the HTTP request will look like
implementing the trait `Api` for `Delay`.
We'll send a `GET` request to `/delay/:n` and `:n` will be
replaced with the value in `Delay.delay`.

```rust
impl api::Api for Delay {
    type Reply = Info;
    type Body = io::Empty;
    type Error = serde_json::Error;

    fn method(&self) -> api::Method {
        api::Method::Get
    }

    fn path(&self) -> String {
        // use a safe joiner to create the path!
        format!("/delay/{}", self.delay)
    }

    fn query(&self) -> api::Query {
        // we'll send any parameter in the query
        Query::new()
    }

    fn headers(&self) -> api::Headers {
        let mut headers = api::Headers::new();

        headers.insert("X-Request-ID".to_string(), vec!["abcde".to_string()]);

        headers
    }

    fn body(&self) -> std::io::Empty {
       std::io::empty()
    }
}
```

Now, we need to create the reply `Info` from
the HTTP response represented by the trait `HttpResponse`.

```rust
impl api::Api for Delay {
    type Reply = Info;
    type Body = io::Empty;
    type Error = serde_json::Error;

    ...

    fn parse<R: HttpResponse>(&self, resp: &mut R) -> Result<Info, serde_json::Error> {
        serde_json::from_reader(resp.body())
    }
}
```

`api` has a trait `Client` to create an adapter for the actual HTTP client, and it implements it for `hyper::Client`.
`Client` has a method `send` that accepts the base URL for the API
and the request.

```rust
extern crate hyper;

use api::Client;

...

fn main() {
    let mut client = hyper::Client::new();

    let resp = client.send("http://httpbin.org/", Delay { delay: 1 });

    println!("{:?}", resp);
}
```

The full code is in `examples/httpbin.rs`
(run `cargo run --example=httpbin --features=use-hyper`).

[httpbin]: https://httpbin.org/ "httpbin"
