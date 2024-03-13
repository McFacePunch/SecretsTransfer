// test webserver's apis

use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::IntoResponse,
    extract::Form,
};
use super::api;  // Assuming 'api.rs' is directly in 'src', adjust if needed 

//use axum_test::TestServer;
//use hyper::body::to_bytes;

//axum tobytes
use axum::body::to_bytes;
use super::api::SignupForm; 
use serde::Serialize;
use axum::http; // Add this line




//use serde_urlencoded;

#[cfg(test)]
mod tests {
    use super::*; // Use items from the parent module 

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

#[tokio::test]
async fn test_signup_get_handler() {
    // Since this handler doesn't have dependencies, you can test it directly
    let response = api::signup_get_handler().await.into_response();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(),1024).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Detailed assertions on the HTML structure
    assert!(body_str.contains("<h1>Signup</h1>"));
    assert!(body_str.contains("<form id=\"signup-form\""));
    // ... Add more specific assertions for the expected structure
}

    #[tokio::test]
    async fn test_signup_post_handler_invalid_email() {
        let signup_data = SignupForm {
            email: "invalid_email_format".to_string(),
            password: "strongpassword".to_string(),
        };

        let request = Request::builder()
            .method("POST")
            .uri("/signup")
            .header(http::header::CONTENT_TYPE, "application/x-www-form-urlencoded") // Optional, may be set automatically by Axum
            .body(Body::from(serde_urlencoded::to_string(&signup_data).unwrap())) // Convert SignupForm to URL-encoded string
            .unwrap();

        let response = api::signup_post_handler(Form(signup_data)).await;
        assert_eq!(response, StatusCode::BAD_REQUEST); 
    }

    #[tokio::test]
    async fn test_signup_post_handler_valid_email() {
        let signup_data = SignupForm {
            email: "wdwdw@wdawd.co".to_string(),
            password: "strongpassword".to_string(),
        };

        let request = Request::builder()
            .method("POST") 
            .uri("/signup")
            .header(http::header::CONTENT_TYPE, "application/x-www-form-urlencoded") // Add this line
            .body(Body::from(serde_urlencoded::to_string(&signup_data).unwrap()))
            .unwrap();

        let response = api::signup_post_handler(Form(signup_data)).await;
        assert_eq!(response, StatusCode::BAD_REQUEST); 
    }






}