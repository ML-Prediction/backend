use serde::{Deserialize, Serialize};
use crate::auth::Usuario;
use crate::otimizacao::{PedidoOtimizacao, PedidoNovaDistancia, DadosPrevisao};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub nome: String,
    pub senha: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub usuario: UsuarioResponse,
    pub mensagem: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthCredentials {
    pub nome: String,
    pub senha: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsuarioResponse {
    pub id: u32,
    pub nome: String,
    pub perfil: String,
}

impl From<&Usuario> for UsuarioResponse {
    fn from(usuario: &Usuario) -> Self {
        UsuarioResponse {
            id: usuario.id,
            nome: usuario.nome.clone(),
            perfil: usuario.perfil.as_str().to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CriarUsuarioRequest {
    pub nome: String,
    pub senha: String,
    pub perfil: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InserirColetaRequest {
    pub nome: String,
    pub senha: String,
    pub tipo: String,
    pub quantidade: f32,
    pub observacoes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PredicaoRequest {
    pub nome: String,
    pub senha: String,
    pub tipo: String,
    pub quantidade: f32,
    pub observacoes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CriarUsuarioRequestComAuth {
    pub nome: String,
    pub senha: String,
    pub novo_usuario: CriarUsuarioRequest,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OtimizarRotaRequest {
    pub nome: String,
    pub senha: String,
    pub pedido: PedidoOtimizacao,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdicionarDistanciaRequest {
    pub nome: String,
    pub senha: String,
    pub distancia: PedidoNovaDistancia,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AtualizarPrevisaoRequest {
    pub nome: String,
    pub senha: String,
    pub previsao: DadosPrevisao,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PredicaoResponse {
    pub predicao: crate::predicao::Predicao,
    pub analise_ia: String,
    pub co2_estimado: Co2Estimado,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Co2Estimado {
    pub tipo: String,
    pub quantidade: f32,
    pub fator: f32,
    pub co2_evitado: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        ApiResponse {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    pub fn error(message: String) -> Self {
        ApiResponse {
            success: false,
            data: None,
            message: Some(message),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

