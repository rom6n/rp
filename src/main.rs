use axum::{body::Body, extract::Request, http::Uri, handler, middleware, routing::{delete, get, post, put}, Router};
use env_logger;
use tower::service_fn;
use tower_http::{services::ServeFile, trace::TraceLayer, compression::CompressionLayer};
use http::Response;
use std::{convert::Infallible, io};

mod handlers;
use handlers::*;


#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let app_data = 10111001;

    let first_page = Router::new().route("/", get(main_page).post(main_page).delete(main_page).put(main_page).with_state(app_data));
    let register = Router::<()>::new().route("/reg/{name}/{email}/{password}", get(|uri: Uri| async move {format!("/reg is not working. Uri: {}", uri)}).with_state(app_data));
    let greet = Router::<()>::new().route("/{name}", get(greet).with_state(app_data));

    let files = Router::<()>::new()
                            .route_service("/toml", ServeFile::new("Cargo.toml"))
                            .route_service("/static", ServeFile::new("static"));

    let routes = Router::<()>::new()
                .merge(first_page)
                .merge(register)
                .merge(greet)
                ;

    let app = Router::<()>::new()
                .without_v07_checks()
                .merge(routes)
                .merge(files)

                .route_service("/i", service_fn(|req: Request| async move {
                    let body = Body::from(format!("/i page, method: {}", req.method()));
                    let res = Response::new(body);
                    Ok::<_, Infallible>(res)

                }))
            
                .fallback(fallback)
                .layer(CompressionLayer::new())
                .layer(TraceLayer::new_for_http())
                ;
            
                
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap()
}