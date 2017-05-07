#![cfg(feature = "use-hyper")]
extern crate api;
extern crate hyper;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::io;
use api::Client;


struct Delay {
    delay: u8
}


#[derive(Debug, Deserialize)]
struct Info {
    origin: String,
    url: String,
}

impl api::Api for Delay {
    type Reply = Info;
    type Body = io::Empty;
    type Error = serde_json::Error;

    fn method(&self) -> api::Method {
        api::Method::Get
    }

    fn path(&self) -> String {
        format!("/delay/{}", self.delay)
    }

    fn query(&self) -> api::Query {
        api::Query::new()
    }

    fn headers(&self) -> api::Headers {
        api::Headers::new()
    }

    fn body(&self) -> io::Empty {
        io::empty()
    }

    fn parse<H>(&self, resp: &mut H) -> Result<Info, serde_json::Error>
        where H: api::HttpResponse
    {
        serde_json::from_reader(resp.body())
    }
}

fn main() {
    let mut client = hyper::Client::new();

    let resp = client.send("http://httpbin.org/", Delay { delay: 1 });

    println!("{:?}", resp);
}
