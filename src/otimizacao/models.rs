// src/otimizacao/models.rs
use serde::{Deserialize, Serialize};

// --- Structs de Resposta (Públicas) ---

#[derive(Serialize, Debug, Clone)]
pub struct MetricasDeCusto {
    pub distancia_total_km: f64,
    pub litros_consumidos: f64,
    pub custo_financeiro_reais: f64,
}

#[derive(Serialize, Debug, Clone)]
pub struct RotaDetalhada {
    pub tipo_otimizacao: String,
    pub sequencia_pontos: Vec<String>,
    pub metricas: MetricasDeCusto,
}

#[derive(Serialize, Debug)]
pub struct BenchmarkInfo {
    pub consumo_medio_kml: f64,
    pub preco_diesel_reais_litro: f64,
}

#[derive(Serialize, Debug)]
pub struct ComparacaoOtimizacao {
    pub rota_gulosa: RotaDetalhada,
    pub rota_prioridade: RotaDetalhada,
    pub benchmark_usado: BenchmarkInfo,
}

// --- Struct Interna (Privada para o módulo) ---
#[derive(Debug, Clone)]
pub(crate) struct ResultadoRotaInterna {
    pub sequencia_pontos: Vec<String>,
    pub distancia_total_km: f64,
}

// --- Structs de Pedido (Públicas) ---

#[derive(Deserialize, Serialize, Debug)]
pub struct PedidoOtimizacao {
    pub garagem_id: String,
    pub pontos_a_visitar: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PedidoNovaDistancia {
   pub origem: String,
   pub destino: String,
   pub custo: f64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DadosPrevisao {
    pub ponto_id: String,
    pub regiao: String,
    pub previsao_demanda: f64,
}