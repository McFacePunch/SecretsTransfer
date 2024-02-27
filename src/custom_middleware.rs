use axum::{
    body::{Body, Bytes},
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use axum::extract::Extension;
use axum::extract::ConnectInfo;
use axum::extract::FromRequest;

use http_body_util::BodyExt;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};



pub async fn print_request_response(Extension(debug_mode): Extension<bool>, ConnectInfo(remote_addr): ConnectInfo<SocketAddr>, req: Request, next: Next) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO pass request into buffer_and_print for logging

    if debug_mode {
        let method = req.method().clone();
        let uri = req.uri().clone();
        let headers = req.headers().clone();

        // Use `tracing` for structured logging
        tracing::debug!(
            method = %method,
            uri = %uri,
            headers = ?headers,
            remote_address = %remote_addr, 
            "Received a request"
        );
    
        let (parts, body) = req.into_parts();
        let bytes = buffer_and_print("request", body).await?;
        let req = Request::from_parts(parts, Body::from(bytes));

        let res = next.run(req).await;

        let (parts, body) = res.into_parts();
        let bytes = buffer_and_print("response", body).await?;
        let res = Response::from_parts(parts, Body::from(bytes));
        return Ok(res);
    } else {

        let res = next.run(req).await;
        return Ok(res)
    }
    //next.run(req).await
}

async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {direction} body: {err}"),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::debug!("{direction} body = {body:?}");
    }

    Ok(bytes)
}

/*pub async fn log_request(
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next, 
) -> Result<Response<Body>, BoxError> {

    // Extract the information
    let method = request.method().clone();
    let uri = request.uri().clone();
    let headers = request.headers().clone();

    // Use `tracing` for structured logging
    info!(
        method = %method,
        uri = %uri,
        headers = ?headers,
        remote_address = %remote_addr, 
        "Received a request"
    );

    let response = next.run(request).await;

    Ok(response)
}*/



/*pub async fn auth_middleware(
    Extension(user): Extension<User>, 
    next: middleware::Next<Body>
) -> Result<impl Response, MyCustomError> {
    if user.is_authenticated() {
        Ok(next.run().await) 
    } else {
        Err(MyCustomError::Unauthorized) 
    }
}*/