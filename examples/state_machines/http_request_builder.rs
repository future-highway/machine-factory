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

use machine_factory::deterministic_state_machine;
use reqwest::{header::HeaderMap, Method, Url};

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

impl HttpRequestStateTrait for Response {}

deterministic_state_machine!(pub HttpRequest {
    context: HttpRequestBuilderData,
    state_trait: trait HttpRequestStateTrait {},
    // This is an optional field
    acceptor_trait: HttpRequestAcceptorTrait,
    // All transitions functions are implemented in this struct, either directly or through the acceptor_trait (if provided)
    acceptor_struct: HttpRequestAcceptor,
    error: Box<&'static str>,
    transitions: [
        NeedsMethod.set_method(method: Method) ->
        NeedsUrl.set_url(url: Url) ->
        NeedsHeaders.set_headers(headers: HeaderMap) ->
        NeedsBody.set_body(body: Option<Vec<u8>>)? ->
        Pending.send().await?reqwest::Error -> Response
    ]
});

#[allow(dead_code)]
struct HttpRequestAcceptor;

impl HttpRequestAcceptorTrait for HttpRequestAcceptor {
    fn set_method(
        state: HttpRequest<NeedsMethod>,
        method: Method,
    ) -> HttpRequest<NeedsUrl> {
        let HttpRequest { mut context, .. } = state;
        context.method = Some(method);
        HttpRequest { context, state: NeedsUrl }
    }

    fn set_url(
        state: HttpRequest<NeedsUrl>,
        url: Url,
    ) -> HttpRequest<NeedsHeaders> {
        let HttpRequest { mut context, .. } = state;
        context.url = Some(url);
        HttpRequest { context, state: NeedsHeaders }
    }

    fn set_headers(
        state: HttpRequest<NeedsHeaders>,
        head: HeaderMap,
    ) -> HttpRequest<NeedsBody> {
        let HttpRequest { mut context, .. } = state;
        context.headers = Some(head);
        HttpRequest { context, state: NeedsBody }
    }

    fn set_body(
        state: HttpRequest<NeedsBody>,
        body: Option<Vec<u8>>,
    ) -> Result<HttpRequest<Pending>, Box<&'static str>>
    {
        if let Some(body) = &body {
            if body.len() > 1024 {
                return Err(Box::new("body too large"));
            }
        }

        let HttpRequest { mut context, .. } = state;
        context.body = body;
        Ok(HttpRequest { context, state: Pending })
    }

    #[allow(dead_code)]
    async fn send(
        state: HttpRequest<Pending>,
    ) -> Result<HttpRequest<Response>, reqwest::Error> {
        let HttpRequest { mut context, .. } = state;
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

        Ok(HttpRequest {
            context,
            state: Response { status_code },
        })
    }
}

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
