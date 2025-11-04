use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use std::collections::HashMap;
use chrono::Utc;

use crate::auth::{Usuario, PerfilUsuario};
use crate::api::routes::AppState;
use crate::api::models::*;
use crate::dataset::Dataset;
use crate::modelo::ModeloML;
use crate::predicao::Predicao;
use crate::otimizacao::PedidoNovaDistancia;
use crate::ia_api;
use std::path::Path as StdPath;

// Helper para validar credenciais
fn validar_usuario(conn: &std::sync::MutexGuard<rusqlite::Connection>, nome: &str, senha: &str) -> Result<Usuario, String> {
    match conn.prepare("SELECT id, nome, perfil, password_hash FROM usuarios WHERE nome = ?1") {
        Ok(mut stmt) => {
            match stmt.query_row([nome], |row| {
                let id: u32 = row.get(0)?;
                let nome_db: String = row.get(1)?;
                let perfil_str: String = row.get(2)?;
                let hash: String = row.get(3)?;
                
                let perfil = PerfilUsuario::try_from(perfil_str.as_str())
                    .map_err(|_| rusqlite::Error::InvalidColumnType(2, "perfil".to_string(), rusqlite::types::Type::Text))?;
                
                Ok((Usuario { id, nome: nome_db, perfil }, hash))
            }) {
                Ok((usuario, hash)) => {
                    if bcrypt::verify(senha, &hash).unwrap_or(false) {
                        Ok(usuario)
                    } else {
                        Err("Senha incorreta".to_string())
                    }
                }
                Err(_) => Err("Usuário não encontrado".to_string()),
            }
        }
        Err(_) => Err("Erro ao acessar banco de dados".to_string()),
    }
}

// ========== AUTENTICAÇÃO ==========

pub async fn login(
    State(app_state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, StatusCode> {
    let conn = app_state.auth.conn.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let usuario = match validar_usuario(&conn, &payload.nome, &payload.senha) {
        Ok(u) => u,
        Err(e) => return Ok(Json(ApiResponse::error(e))),
    };

    let response = LoginResponse {
        usuario: UsuarioResponse::from(&usuario),
        mensagem: "Login realizado com sucesso! Use suas credenciais nos próximos requests.".to_string(),
    };

    Ok(Json(ApiResponse::success(response)))
}

pub async fn criar_usuario(
    State(app_state): State<AppState>,
    Json(payload): Json<crate::api::models::CriarUsuarioRequestComAuth>,
) -> Result<Json<ApiResponse<UsuarioResponse>>, StatusCode> {
    // Validar credenciais do usuário atual
    let conn = app_state.auth.conn.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let usuario_atual = match validar_usuario(&conn, &payload.nome, &payload.senha) {
        Ok(u) => u,
        Err(e) => return Ok(Json(ApiResponse::error(format!("Credenciais inválidas: {}", e)))),
    };

    if !usuario_atual.pode_gerenciar_usuarios() {
        return Ok(Json(ApiResponse::error("Acesso negado. Apenas administradores podem criar usuários.".to_string())));
    }

    let perfil = match PerfilUsuario::try_from(payload.novo_usuario.perfil.as_str()) {
        Ok(p) => p,
        Err(_) => return Ok(Json(ApiResponse::error("Perfil inválido".to_string()))),
    };

    let password_hash = bcrypt::hash(&payload.novo_usuario.senha, bcrypt::DEFAULT_COST)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match conn.execute(
        "INSERT INTO usuarios (nome, perfil, password_hash) VALUES (?1, ?2, ?3)",
        (&payload.novo_usuario.nome, perfil.as_str(), password_hash),
    ) {
        Ok(_) => {
            // Buscar usuário criado
            let id = conn.last_insert_rowid() as u32;
            let novo_usuario = Usuario {
                id,
                nome: payload.novo_usuario.nome.clone(),
                perfil,
            };
            Ok(Json(ApiResponse::success(UsuarioResponse::from(&novo_usuario))))
        }
        Err(e) => Ok(Json(ApiResponse::error(format!("Erro ao criar usuário: {}", e)))),
    }
}

// ========== COLETA ==========

pub async fn inserir_coleta(
    State(app_state): State<AppState>,
    Json(payload): Json<InserirColetaRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let conn = app_state.auth.conn.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let usuario = match validar_usuario(&conn, &payload.nome, &payload.senha) {
        Ok(u) => u,
        Err(e) => return Ok(Json(ApiResponse::error(format!("Credenciais inválidas: {}", e)))),
    };

    if !usuario.pode_inserir_dados() {
        return Ok(Json(ApiResponse::error("Acesso negado".to_string())));
    }

    let db_path = StdPath::new("data/db.json");
    let mut dataset = Dataset::load_from_file(db_path)
        .unwrap_or_else(|_| Dataset::new());
    
    dataset.add_entry(payload.tipo.clone(), payload.quantidade, payload.observacoes);
    
    dataset.save_to_file(db_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success("Dados inseridos com sucesso".to_string())))
}

pub async fn listar_coletas(
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<crate::dataset::WasteEntry>>>, StatusCode> {
    let db_path = StdPath::new("data/db.json");
    let dataset = Dataset::load_from_file(db_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut entries = dataset.entries;

    // Filtrar por tipo se fornecido
    if let Some(tipo) = params.get("tipo") {
        entries.retain(|e| e.tipo.eq_ignore_ascii_case(tipo));
    }

    // Paginação
    let limit: usize = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(100);
    let offset: usize = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);
    
    let paginated = entries.into_iter().skip(offset).take(limit).collect();

    Ok(Json(ApiResponse::success(paginated)))
}

// ========== PRÉ-PROCESSAMENTO ==========

pub async fn executar_preprocessamento(
    State(app_state): State<AppState>,
    Json(creds): Json<AuthCredentials>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let conn = app_state.auth.conn.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let usuario = match validar_usuario(&conn, &creds.nome, &creds.senha) {
        Ok(u) => u,
        Err(e) => return Ok(Json(ApiResponse::error(format!("Credenciais inválidas: {}", e)))),
    };

    if !usuario.pode_pre_processar() {
        return Ok(Json(ApiResponse::error("Acesso negado".to_string())));
    }

    // Simular pré-processamento
    Ok(Json(ApiResponse::success("Pré-processamento executado com sucesso".to_string())))
}

// ========== PREDIÇÕES ==========

pub async fn executar_predicao(
    State(app_state): State<AppState>,
    Json(payload): Json<PredicaoRequest>,
) -> Result<Json<ApiResponse<crate::api::models::PredicaoResponse>>, StatusCode> {
    // Validar usuário e liberar o guard antes de qualquer await
    let usuario = {
        let conn = app_state.auth.conn.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        match validar_usuario(&conn, &payload.nome, &payload.senha) {
            Ok(u) => u,
            Err(e) => return Ok(Json(ApiResponse::error(format!("Credenciais inválidas: {}", e)))),
        }
    };

    if !usuario.pode_acessar_predicoes() {
        return Ok(Json(ApiResponse::error("Acesso negado".to_string())));
    }

    let db_path = StdPath::new("data/db.json");
    let mut dataset = Dataset::load_from_file(db_path)
        .unwrap_or_else(|_| Dataset::new());

    dataset.add_entry(payload.tipo.clone(), payload.quantidade, payload.observacoes.clone());
    dataset.save_to_file(db_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut modelo = ModeloML::new("ModeloSimuladoReciclagem");
    modelo.treinar(&dataset);
    let _ = modelo.salvar("output/modelo.json");

    let predicao: Predicao = modelo.prever(&dataset);
    let _ = predicao.exportar();

    // Gerar análise com IA (agora podemos fazer await porque o guard foi liberado)
    let prompt = format!(
        "O usuário coletou {:.3} kg de {}. Considerando os dados históricos, forneça uma BREVE previsão ilustrativa sobre impacto ambiental e tendências. Responda em até 50 palavras.",
        payload.quantidade, payload.tipo
    );

    let analise_ia = ia_api::gerar_resposta_preditiva(&prompt).await
        .unwrap_or_else(|_| "Erro ao gerar análise com IA".to_string());

    let fator = co2_factor(&payload.tipo);
    let response = crate::api::models::PredicaoResponse {
        predicao,
        analise_ia,
        co2_estimado: crate::api::models::Co2Estimado {
            tipo: payload.tipo,
            quantidade: payload.quantidade,
            fator,
            co2_evitado: payload.quantidade * fator,
        },
    };

    Ok(Json(ApiResponse::success(response)))
}

fn co2_factor(tipo: &str) -> f32 {
    match tipo.to_lowercase().as_str() {
        "plastico" | "plástico" => 2.0,
        "papel" => 1.2,
        "vidro" => 0.6,
        "metal" => 3.0,
        "organico" | "orgânico" => 0.3,
        _ => 1.0,
    }
}

// ========== OTIMIZAÇÃO ==========

pub async fn otimizar_rota(
    State(app_state): State<AppState>,
    Json(payload): Json<crate::api::models::OtimizarRotaRequest>,
) -> Result<Json<ApiResponse<crate::otimizacao::ComparacaoOtimizacao>>, StatusCode> {
    let conn = app_state.auth.conn.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let usuario = match validar_usuario(&conn, &payload.nome, &payload.senha) {
        Ok(u) => u,
        Err(e) => return Ok(Json(ApiResponse::error(format!("Credenciais inválidas: {}", e)))),
    };

    if !usuario.pode_otimizar_rotas() {
        return Ok(Json(ApiResponse::error("Acesso negado".to_string())));
    }

    let comparacao = crate::otimizacao::executar_otimizacao_comparativa(&app_state.otimizacao, &payload.pedido);
    Ok(Json(ApiResponse::success(comparacao)))
}

pub async fn adicionar_distancia(
    State(app_state): State<AppState>,
    Json(payload): Json<crate::api::models::AdicionarDistanciaRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let conn = app_state.auth.conn.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let usuario = match validar_usuario(&conn, &payload.nome, &payload.senha) {
        Ok(u) => u,
        Err(e) => return Ok(Json(ApiResponse::error(format!("Credenciais inválidas: {}", e)))),
    };

    if !usuario.pode_otimizar_rotas() {
        return Ok(Json(ApiResponse::error("Acesso negado".to_string())));
    }

    crate::otimizacao::alimentar_distancia(&app_state.otimizacao, payload.distancia.clone());
    
    // Adicionar rota reversa
    let reverso = PedidoNovaDistancia {
        origem: payload.distancia.destino,
        destino: payload.distancia.origem,
        custo: payload.distancia.custo,
    };
    crate::otimizacao::alimentar_distancia(&app_state.otimizacao, reverso);

    Ok(Json(ApiResponse::success("Distância adicionada com sucesso".to_string())))
}

pub async fn atualizar_previsao_demanda(
    State(app_state): State<AppState>,
    Json(payload): Json<crate::api::models::AtualizarPrevisaoRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let conn = app_state.auth.conn.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let usuario = match validar_usuario(&conn, &payload.nome, &payload.senha) {
        Ok(u) => u,
        Err(e) => return Ok(Json(ApiResponse::error(format!("Credenciais inválidas: {}", e)))),
    };

    if !usuario.pode_otimizar_rotas() {
        return Ok(Json(ApiResponse::error("Acesso negado".to_string())));
    }

    crate::otimizacao::alimentar_previsao(&app_state.otimizacao, payload.previsao);
    Ok(Json(ApiResponse::success("Previsão atualizada com sucesso".to_string())))
}

// ========== USUÁRIOS ==========

pub async fn listar_usuarios(
    State(app_state): State<AppState>,
    Json(creds): Json<AuthCredentials>,
) -> Result<Json<ApiResponse<Vec<UsuarioResponse>>>, StatusCode> {
    // Validar usuário e liberar o lock antes de fazer a query
    let usuario = {
        let conn = app_state.auth.conn.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        match validar_usuario(&conn, &creds.nome, &creds.senha) {
            Ok(u) => u,
            Err(e) => return Ok(Json(ApiResponse::error(format!("Credenciais inválidas: {}", e)))),
        }
    };

    if !usuario.pode_gerenciar_usuarios() {
        return Ok(Json(ApiResponse::error("Acesso negado".to_string())));
    }

    // Agora podemos adquirir o lock novamente para a query
    let conn = app_state.auth.conn.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut stmt = conn.prepare("SELECT id, nome, perfil FROM usuarios ORDER BY id")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let usuarios: Vec<UsuarioResponse> = stmt.query_map([], |row| {
        let id: u32 = row.get(0)?;
        let nome: String = row.get(1)?;
        let perfil_str: String = row.get(2)?;
        Ok(UsuarioResponse {
            id,
            nome,
            perfil: perfil_str,
        })
    })
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .filter_map(|r| r.ok())
    .collect();

    Ok(Json(ApiResponse::success(usuarios)))
}

pub async fn deletar_usuario(
    State(app_state): State<AppState>,
    Path(id): Path<u32>,
    Json(creds): Json<AuthCredentials>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // Para DELETE, as credenciais vão no body também
    // Validar usuário e liberar o lock antes de fazer a query
    let usuario = {
        let conn = app_state.auth.conn.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        match validar_usuario(&conn, &creds.nome, &creds.senha) {
            Ok(u) => u,
            Err(e) => return Ok(Json(ApiResponse::error(format!("Credenciais inválidas: {}", e)))),
        }
    };

    if !usuario.pode_gerenciar_usuarios() {
        return Ok(Json(ApiResponse::error("Acesso negado".to_string())));
    }

    if id == usuario.id {
        return Ok(Json(ApiResponse::error("Você não pode deletar a si mesmo".to_string())));
    }

    // Agora podemos adquirir o lock novamente para a query
    let conn = app_state.auth.conn.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match conn.execute("DELETE FROM usuarios WHERE id = ?1", [id]) {
        Ok(0) => Ok(Json(ApiResponse::error("Usuário não encontrado".to_string()))),
        Ok(_) => Ok(Json(ApiResponse::success("Usuário deletado com sucesso".to_string()))),
        Err(e) => Ok(Json(ApiResponse::error(format!("Erro: {}", e)))),
    }
}

// ========== HEALTH ==========

pub async fn health_check() -> Json<ApiResponse<serde_json::Value>> {
    Json(ApiResponse::success(serde_json::json!({
        "status": "ok",
        "timestamp": Utc::now().to_rfc3339()
    })))
}

