use axum::{
    routing::{get, post, delete},
    Router,
};
use std::sync::Arc;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    classify::ServerErrorsFailureClass,
};
use tracing::Level;

use crate::api::handlers::*;
use crate::api::middleware::AuthState;
use crate::otimizacao::EstadoOtimizacao;
use rusqlite::Connection;

pub fn create_router(
    conn: Connection,
    estado_otim: Arc<EstadoOtimizacao>,
    secret: String,
) -> Router {
    let auth_state = AuthState::new(conn, secret);

    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/auth/login", post(login));

    let protected_routes = Router::new()
        .route("/auth/usuarios", post(criar_usuario))
        .route("/coleta", post(inserir_coleta))
        .route("/coleta", get(listar_coletas))
        .route("/preprocessamento/executar", post(executar_preprocessamento))
        .route("/predicoes", post(executar_predicao))
        .route("/otimizacao/rotas", post(otimizar_rota))
        .route("/otimizacao/distancias", post(adicionar_distancia))
        .route("/otimizacao/previsao-demanda", post(atualizar_previsao_demanda))
        .route("/usuarios", post(listar_usuarios))
        .route("/usuarios/:id", delete(deletar_usuario));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    tracing::span!(
                        Level::INFO,
                        "request",
                        method = %request.method(),
                        uri = %request.uri(),
                        version = ?request.version(),
                    )
                })
                .on_request(|request: &axum::http::Request<_>, _span: &tracing::Span| {
                    tracing::info!(
                        "📥 Requisição recebida: {} {}",
                        request.method(),
                        request.uri()
                    );
                })
                .on_response(|_response: &axum::http::Response<_>, latency: std::time::Duration, _span: &tracing::Span| {
                    tracing::info!(
                        "📤 Resposta enviada: status={} latency={:.2}ms",
                        _response.status(),
                        latency.as_secs_f64() * 1000.0
                    );
                })
                .on_failure(|_error: ServerErrorsFailureClass, _latency: std::time::Duration, _span: &tracing::Span| {
                    tracing::error!(
                        "❌ Erro na requisição: {:?} latency={:.2}ms",
                        _error,
                        _latency.as_secs_f64() * 1000.0
                    );
                })
        )
        .layer(CorsLayer::permissive())
        .with_state(AppState {
            auth: auth_state,
            otimizacao: estado_otim,
        })
}

#[derive(Clone)]
pub struct AppState {
    pub auth: AuthState,
    pub otimizacao: Arc<EstadoOtimizacao>,
}

