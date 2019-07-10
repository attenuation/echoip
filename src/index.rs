use futures::{future, Future, Stream};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn, service_fn_ok};
use hyper::{header, Body, Chunk, Method, StatusCode, Uri};
use hyper::{Client, Request, Response, Server};
use serde_json;
use std::net::SocketAddr;
use log::*;
use serde::{Serialize, Deserialize};

use futures::future::IntoFuture;

use crate::btapi;

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type ResponseFuture = Box<dyn Future<Item = Response<Body>, Error = GenericError> + Send>;

#[derive(Serialize, Deserialize, Debug)]
struct Mtrdata {
    service: String,
    ip: String,
}

pub fn index_post(req: Request<Body>, remote_addr: String) -> ResponseFuture {
    Box::new(req.into_body()
        .concat2() // Concatenate all chunks in the body
        .from_err()
        .and_then(|entire_body| {
            // TODO: Replace all unwraps with proper error handling
            println!("hello");
            let str = String::from_utf8(entire_body.to_vec())?;
            dbg!(&str);
            let mut data : Mtrdata = serde_json::from_str(&str).map_err(move |e| {
                error!("error json post from : {}",remote_addr);
                e
            })?;
            let res = match data.service.as_ref() {
                "bt" => {
                    btapi::bt_api_req(data.ip).map(move |web_res| {
                        let body = Body::wrap_stream(web_res.into_body().map(move |b| {
                            let data: btapi::Btdata = serde_json::from_slice(&b).unwrap();
                            let ip_data = btapi::Ipdata::new(data);
                            let json = serde_json::to_string(&ip_data).unwrap();
                            Chunk::from(json)
                        }));
                        let response = Response::builder()
                            .status(StatusCode::OK)
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(body).unwrap();
                        response
                    })
                },
                _ => {
                        let response = Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(Body::empty()).unwrap();
                        future::ok(response)
                }
            };
            res
            // let json = serde_json::to_string(&data)?;

        })
    )
}
