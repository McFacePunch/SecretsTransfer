use axum::{
    body::{Body, Bytes},
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum::extract::Extension;
use axum::extract::ConnectInfo;
//use axum::Error;

use tower::BoxError;



use http_body_util::BodyExt;

use std::net::SocketAddr;
//use std::error::Error;

//use tracing_subscriber::{layer::{Context, SubscriberExt}, util::SubscriberInitExt};


//struct CatchAllErrorMiddleware;


pub async fn print_request_response(Extension(debug_requests): Extension<bool>,
 ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
  req: Request, next: Next) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO pass request into buffer_and_print for logging

    if debug_requests {
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


#[allow(dead_code)]
//pub async fn handle_timeout_error(err: BoxError,) -> (StatusCode, String) {
pub async fn handle_timeout_error(err: Box<dyn std::error::Error + Send + Sync + 'static>) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Failed with {err}"),
    )
}

#[allow(dead_code)]
pub async fn handle_error(err: BoxError, Extension(debug_requests): Extension<bool>,  ConnectInfo(remote_addr): ConnectInfo<SocketAddr>, req: Request, next: Next) -> Result<Response<Body>, BoxError> {
    // Log the error and option verbose logging under debug mode
    tracing::error!("Error: {}.", err);// Caused by: {}", err, err.source().unwrap_or_default);
    let response;

    if debug_requests {
        // debug mode so return full info
        response = Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("Internal Server Error"))
            .unwrap();
    } else {
        // return a "404 Not Found" and log the actual error
        // mimic this: (StatusCode::NOT_FOUND, "404, Not Found")
        response = Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("404, Not Found"))
            .unwrap();
    }

    let method = req.method().clone();
    let uri = req.uri().clone();
    let headers = req.headers().clone();

    let (parts, body) = req.into_parts();
    let bytes = buffer_and_print("request", body).await;
    let bytes = match bytes {
        Ok(bytes) => bytes,
        Err((status, message)) => {
            // Handle the error here
            // For example, you can log the error and return an empty response
            tracing::error!("Error buffering request: {}: {}", status, message);
            return Ok(Response::new(Body::empty()));
        }
    };
    let req = Request::from_parts(parts, Body::from(bytes));

    tracing::error!(
        method = %method,
        uri = %uri,
        headers = ?headers,
        remote_address = %remote_addr,
        "Request caused an error:\n"
    );

    // Return the response
    let _res = next.run(req).await;//todo fix response
    //return Ok(res)
    return Ok(response)
}


/*impl<S> Service<S> for CatchAllErrorMiddleware {
    type Response = Response;
    type Error = BoxError; 
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_, MyService>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, target: S) -> Self::Future {
        future::ready(target.oneshot(Request::default()).map_err(|err| err.into()))
    }
}

// Generic error handling function
async fn handle_generic_error(_err: Box<dyn Error + Send + Sync + 'static>) -> impl IntoResponse {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "A generic error occurred".to_string(),
    )
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