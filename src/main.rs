use axum::{Router, middleware, routing::get};
use tower_governor::{
    GovernorLayer, governor::GovernorConfigBuilder, key_extractor::GlobalKeyExtractor,
};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;
use tracing_subscriber::EnvFilter;
use umfeld::{AppState, index, rate_limit_response, require_place, snapshot};

const API_KEY_NAME: &str = "TANKER_API_KEY";
const PORT_NAME: &str = "PORT";
const DEFAULT_PORT: u16 = 3000;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "umfeld=info,tower_http=info".into()),
        )
        .init();

    let state = AppState {
        fuel_api_key: std::env::var(API_KEY_NAME)
            .unwrap_or_else(|_| panic!("{API_KEY_NAME} must be set"))
            .into(),
        web_client: reqwest::Client::new(),
    };

    let governor_conf = GovernorConfigBuilder::default()
        .per_second(60)
        .burst_size(1)
        .key_extractor(GlobalKeyExtractor)
        .finish()
        .unwrap();

    let app = Router::new()
        .route("/snapshot", get(snapshot))
        .route_layer(GovernorLayer::new(governor_conf).error_handler(rate_limit_response))
        // check params before rate limiting
        .route_layer(middleware::from_fn(require_place))
        // not rate limited
        .route("/", get(index))
        .with_state(state)
        // outermost, so rejected and rate limited requests are logged too
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        );

    let port = match std::env::var(PORT_NAME) {
        Ok(port) => port
            .parse()
            .unwrap_or_else(|_| panic!("{PORT_NAME} must be a port number, got {port:?}")),
        Err(_) => DEFAULT_PORT,
    };
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
