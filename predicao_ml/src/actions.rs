use crate::auth::Usuario;
use crate::predicoes_module;
use crate::otimizacao;
use crate::dataset::Dataset; 
use std::path::Path;
use crate::otimizacao::{EstadoOtimizacao, PedidoOtimizacao};
use serde_json;

pub async fn inserir_dados_coleta(
    usuario: &Usuario, 
    tipo: String, 
    quantidade: f32, 
    observacoes: Option<String>
) -> Result<(), String> {
    
    if !usuario.pode_inserir_dados() {
        return Err(format!(
            "❌ ACESSO NEGADO: {} (Perfil: {:?}) não pode inserir dados.",
            usuario.nome, usuario.perfil
        ));
    }
    println!("🔄 Salvando entrada...");

    let db_path = Path::new("data/db.json");

    let mut dataset = Dataset::load_from_file(db_path)
        .unwrap_or_else(|err| {
            println!("Aviso: Não foi possível carregar db.json ({}). Criando um novo.", err);
            Dataset::new()
        });
    dataset.add_entry(tipo.clone(), quantidade, observacoes);

    match dataset.save_to_file(db_path) {
        Ok(_) => {
            println!("✅ SUCESSO: {} (ID: {}) inseriu dados: {} - {}kg", 
                usuario.nome, usuario.id, tipo, quantidade
            );
            Ok(())
        },
        Err(e) => {
            eprintln!("Erro crítico ao salvar db.json: {}", e);
            Err("Falha ao salvar os dados no arquivo.".to_string())
        }
    }
}

pub fn executar_pre_processamento(usuario: &Usuario) -> Result<(), String> {
    if !usuario.pode_pre_processar() {
        return Err(format!(
            "❌ ACESSO NEGADO: {} (Perfil: {:?}) não pode executar o pré-processamento.",
            usuario.nome, usuario.perfil
        ));
    }

    println!("✅ SUCESSO: {} (ID: {}) iniciou o pré-processamento.", usuario.nome, usuario.id);
    Ok(())
}

pub async fn acessar_modulo_predicoes(usuario: &Usuario) -> Result<(), String> {
    if !usuario.pode_acessar_predicoes() {
        return Err(format!(
            "❌ ACESSO NEGADO: {} (Perfil: {:?}) não pode acessar o módulo de predições.",
            usuario.nome, usuario.perfil
        ));
    }
    
if let Err(e) = predicoes_module::run_prediction_module().await {
        eprintln!("Erro no módulo de predição: {}", e);
        return Err("Falha ao executar o módulo de predição.".to_string());
    }

    Ok(())
}

pub fn acessar_modulo_otimizacao(
    usuario: &Usuario, 
    estado: &otimizacao::EstadoOtimizacao
) -> Result<(), String> {
    
    if !usuario.pode_otimizar_rotas() {
        return Err(format!(
            "❌ ACESSO NEGADO: {} (Perfil: {:?}) não pode otimizar rotas.",
            usuario.nome, usuario.perfil
        ));
    }
    
    println!("\n--- 🚛 Módulo de Otimização de Rotas ---");
    
    let pedido_mock = PedidoOtimizacao {
        garagem_id: "garagem".to_string(),
        pontos_a_visitar: vec![
            "ponto_A".to_string(), 
            "ponto_B".to_string(), 
            "ponto_C".to_string()
        ],
    };
    println!("Simulando pedido para os pontos: {:?}", pedido_mock.pontos_a_visitar);

    let comparacao = otimizacao::executar_otimizacao_comparativa(
        estado,
        &pedido_mock
    );

    println!("\n--- ✅ Comparação de Otimização Concluída ---");
    let json_output = serde_json::to_string_pretty(&comparacao)
        .map_err(|e| format!("Erro ao formatar resultado: {}", e))?;
        
    println!("{}", json_output);

    Ok(())
}