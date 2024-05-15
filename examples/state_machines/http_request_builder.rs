//! This example demonstrates how to use the
//! `deterministic_state_machine` macro to create a simple
//! HTTP request builder.
//!
//! State transistions are enfored at compile time.
//!
//! In this example, the state machine gathers all the
//! required data to make an HTTP request, and then
//! sends the request. The user cannot call `send` until all
//! the required data is set.
#![allow(dead_code)]

use anyhow::Context;
use machine_factory::deterministic_state_machine;
use reqwest::{header::HeaderMap, Method, Url};
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct HttpRequestBuilderData {
    pub method: Option<Method>,
    pub url: Option<Url>,
    pub headers: Option<HeaderMap>,
    pub body: Option<Vec<u8>>,
}

pub struct NeedsMethod;
impl HttpRequestStateTrait for NeedsMethod {}

pub struct NeedsUrl;
impl HttpRequestStateTrait for NeedsUrl {}

pub struct NeedsHeaders;
impl HttpRequestStateTrait for NeedsHeaders {}

pub struct NeedsBody;
impl HttpRequestStateTrait for NeedsBody {}

pub struct Pending;
impl HttpRequestStateTrait for Pending {}

pub struct Response {
    pub status_code: u16,
}

deterministic_state_machine!(
    /// A simple HTTP request builder
    #[derive(Debug, Clone)]
    pub HttpRequest {
        context: HttpRequestBuilderData,
        // Optionally specify a trait that all states must implement
        state_trait: trait HttpRequestStateTrait {},
        transitions: [
            NeedsMethod {
                pub fn set_method(self, method: Method) -> HttpRequest<NeedsUrl> {
                    let Self { mut context, .. } = self;
                    context.method = Some(method);
                    HttpRequest { context, state: NeedsUrl }
                }
            },
            NeedsUrl {
                pub fn set_url(self, url: Url) -> HttpRequest<NeedsHeaders> {
                    let Self { mut context, .. } = self;
                    context.url = Some(url);
                    HttpRequest { context, state: NeedsHeaders }
                }
            },
            NeedsHeaders {
                pub fn set_headers(self, headers: HeaderMap) -> HttpRequest<NeedsBody> {
                    let Self { mut context, .. } = self;
                    context.headers = Some(headers);
                    HttpRequest { context, state: NeedsBody }
                }
            },
            NeedsBody {
                pub fn no_body(self) -> HttpRequest<Pending> {
                    self.set_body(None).expect("setting body failed")
                }

                pub fn set_body(self, body: Option<Vec<u8>>) -> Result<HttpRequest<Pending>, &'static str> {
                    if let Some(body) = &body {
                        if body.len() > 1024 {
                            return Err("body too large");
                        }
                    }

                    let HttpRequest { mut context, .. } = self;
                    context.body = body;
                    Ok(HttpRequest { context, state: Pending })
                }

                pub fn set_json_body<Body: Serialize>(self, body: &Body) -> anyhow::Result<HttpRequest<Pending>> {
                    let body = serde_json::to_vec(body).context("failed to serialize body")?;
                    self.set_body(Some(body)).map_err(|e| anyhow::anyhow!(e)).context("failed to set body")
                }

                pub async fn send(self) -> Result<Response, reqwest::Error> {
                    self.no_body().send().await
                }
            },
            Pending {
                pub async fn send(self) -> Result<Response, reqwest::Error> {
                    let HttpRequest { mut context, .. } = self;
                    let HttpRequestBuilderData {
                        method,
                        url,
                        headers,
                        body,
                    } = &mut context;

                    let url = url.take().expect("url not set");

                    let request = reqwest::Client::new();
                    let request = match method
                        .take()
                        .expect("method not set")
                    {
                        Method::GET => request.get(url),
                        Method::POST => request.post(url),
                        Method::PUT => request.put(url),
                        Method::DELETE => request.delete(url),
                        #[allow(clippy::unimplemented)]
                        _ => unimplemented!(
                            "Implement the rest, or make sure the state machine is not used with unsupported methods"
                        ),
                    };

                    let request = request.headers(
                        headers.take().expect("headers not set"),
                    );

                    let request = if let Some(body) = body.take() {
                        request.body(body)
                    } else {
                        request
                    };

                    let response = request.send().await?;
                    let status_code = response.status().as_u16();

                    Ok(Response { status_code })
                }
            },
        ]
});

impl Default for HttpRequest<NeedsMethod> {
    fn default() -> Self {
        Self::new(
            NeedsMethod,
            HttpRequestBuilderData {
                method: None,
                url: None,
                headers: None,
                body: None,
            },
        )
    }
}
