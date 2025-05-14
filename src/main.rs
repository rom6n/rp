use axum::{
    body::Body, response::{IntoResponse, Html, Json, Response, Redirect},
    extract::{OriginalUri, DefaultBodyLimit, Request, State, ConnectInfo},
    http::{Uri, StatusCode}, 
    handler, middleware, 
    routing::{delete, get, post, put}, 
    Router
};
use tokio::time::Duration;
use env_logger;
use tower::{buffer::BufferLayer, layer, limit::ConcurrencyLimitLayer, load_shed::LoadShedLayer, service_fn, spawn_ready, timeout::TimeoutLayer, ServiceBuilder, ServiceExt};
use tower_http::{services::ServeFile, trace::TraceLayer, compression::CompressionLayer};
use std::{convert::Infallible, io, net::SocketAddr};
use tracing::info_span;
use std::sync::Arc;


use rp::models::*;
use rp::handlers::*;


#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let database_pool = Arc::new(DataBase::create_connection().await);
    let redis_pool = Arc::new(Redis::create_connection().await);

    let first_page = Router::new().route("/", get(main_page).post(main_page).delete(main_page).put(main_page));
    let register_page = Router::new().route("/reg/{nickname}/{name}/{password}", get(register).with_state((Arc::clone(&database_pool), Arc::clone(&redis_pool))));
    let profile_page = Router::new().route("/profile/{nickname}", get(profile).with_state((Arc::clone(&database_pool), Arc::clone(&redis_pool))));
    let greet_page = Router::new().route("/{name}", get(greet).with_state(ExampleData {a: 3}));
    let all_users_page = Router::new().route("/all", get(all_users).with_state((Arc::clone(&database_pool), Arc::clone(&redis_pool))));
    let my_profile_page = Router::new().route("/profile", get(my_profile).with_state((Arc::clone(&database_pool), Arc::clone(&redis_pool))));
    let login_page = Router::new().route("/login/{nickname}/{password}", get(login).with_state((Arc::clone(&database_pool), Arc::clone(&redis_pool))));
    let cipher_text_path = Router::new().route("/cipher/{data}", get(cipher_text));
    
    let files = Router::new()
                            .route_service("/toml", ServeFile::new("Cargo.toml"))
                            .route_service("/static", ServeFile::new("static/message.txt"));

    let routes = Router::new()
                .merge(my_profile_page)
                .merge(profile_page)
                .layer(AuthLayer {db_conn: Arc::clone(&database_pool)})
                .merge(first_page)
                .merge(register_page)
                .merge(greet_page)
                .merge(all_users_page)
                .merge(login_page)
                .merge(cipher_text_path)
                ;

    let app = Router::new()
                .without_v07_checks()
                .merge(routes)
                .merge(files)
                //.nest("/", foobar) не используется

                .route_service("/i", service_fn(|req: Request| async move {
                    let body = Body::from(format!("/i page, method: {}", req.method()));
                    let res = Response::new(body);
                    Ok::<_, Infallible>(res)

                }))
                //.route_layer(CompressionLayer::new()) не знаю что делает
                //.fallback_service(service) не знаю что делает
                .fallback(fallback)
                .method_not_allowed_fallback(method_fallback)

                .layer(
                    ServiceBuilder::new()
                        .layer(DefaultBodyLimit::max(4096))
                        .layer(TraceLayer::new_for_http().make_span_with(|_req: &Request<_>| {
                            info_span!("request: ", method = %_req.method(), uri = %_req.uri(), versions = ?_req.version())}))
                        .layer(CompressionLayer::new())
                        .layer(ConcurrencyLimitLayer::new(250))
                        //.layer(BufferLayer::new(500))
                        //.layer(TimeoutLayer::new(Duration::from_secs(15)))
                    )
                ;
            
                
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()/* лучше использовать middleware */).await.unwrap()
}