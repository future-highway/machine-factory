//! This example demonstrates how to use the `HttpRequest`
//! state machine to build and send an HTTP request. See the
//! [`HttpRequest`] state machine for more information.
#![allow(clippy::print_stdout)]

use crate::state_machines::http_request_builder::HttpRequest;
use reqwest::{header::HeaderMap, Method, Url};

mod state_machines;

#[tokio::main]
async fn main() {
    let request = HttpRequest::default();
    let (state, _context) = request
        .set_method(Method::GET)
        .set_url(
            Url::parse("https://example.com")
                .expect("Invalid URL"),
        )
        .set_headers(HeaderMap::new())
        .set_body(None)
        .expect("We didn't set a body")
        .send()
        .await
        .expect("Failed to send request")
        .into_parts();

    println!("Response status code: {}", state.status_code);
}
