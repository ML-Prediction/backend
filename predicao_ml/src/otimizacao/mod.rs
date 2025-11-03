mod models;
mod services;

use services::{ServicoDistancia, ServicoDemanda};
use std::sync::{Arc, Mutex};

pub use models::{
    PedidoOtimizacao, ComparacaoOtimizacao, DadosPrevisao, PedidoNovaDistancia,
    RotaDetalhada, MetricasDeCusto, BenchmarkInfo
};
use models::ResultadoRotaInterna; 


const CONSUMO_CAMINHAO_KML: f64 = 2.0;
const PRECO_DIESEL_REAIS: f64 = 6.0;

#[derive(Clone)]
pub struct EstadoOtimizacao {
    servico_distancia: Arc<Mutex<ServicoDistancia>>,
    servico_demanda: Arc<Mutex<ServicoDemanda>>,
}

impl EstadoOtimizacao {
    pub fn new() -> Self {
        let servico_distancia = ServicoDistancia::new("data/distancias.json"); // Caminho corrigido
        let servico_demanda = ServicoDemanda::new();

        Self {
            servico_distancia: Arc::new(Mutex::new(servico_distancia)),
            servico_demanda: Arc::new(Mutex::new(servico_demanda)),
        }
    }
} 
fn arredondar_duas_casas(num: f64) -> f64 {
    (num * 100.0).round() / 100.0
}

fn calcular_metricas_consumo(distancia_km: f64) -> MetricasDeCusto {
    let litros = distancia_km / CONSUMO_CAMINHAO_KML;
    let reais = litros * PRECO_DIESEL_REAIS;

    MetricasDeCusto {
        distancia_total_km: arredondar_duas_casas(distancia_km),
        litros_consumidos: arredondar_duas_casas(litros),
        custo_financeiro_reais: arredondar_duas_casas(reais),
    }
}

pub fn executar_otimizacao_comparativa(
    estado: &EstadoOtimizacao,
    pedido: &PedidoOtimizacao,
) -> ComparacaoOtimizacao {
    
    println!("Iniciando cálculo de otimização comparativa...");
    
    let servico_dist = estado.servico_distancia.lock().unwrap();
    let servico_dem = estado.servico_demanda.lock().unwrap();

    let rota_gulosa_interna: ResultadoRotaInterna =
        services::otimizar_rota_vizinho_proximo(pedido, &*servico_dist);

    let rota_inteligente_interna: ResultadoRotaInterna =
        services::otimizar_rota_por_prioridade(pedido, &*servico_dist, &*servico_dem);

    let rota_gulosa_detalhada = RotaDetalhada {
        tipo_otimizacao: "Gulosa (Menor Custo)".to_string(),
        sequencia_pontos: rota_gulosa_interna.sequencia_pontos,
        metricas: calcular_metricas_consumo(rota_gulosa_interna.distancia_total_km),
    };

    let rota_inteligente_detalhada = RotaDetalhada {
        tipo_otimizacao: "Prioridade (Demanda Urgente)".to_string(),
        sequencia_pontos: rota_inteligente_interna.sequencia_pontos,
        metricas: calcular_metricas_consumo(rota_inteligente_interna.distancia_total_km),
    };

    let comparacao = ComparacaoOtimizacao {
        rota_gulosa: rota_gulosa_detalhada,
        rota_prioridade: rota_inteligente_detalhada,
        benchmark_usado: BenchmarkInfo {
            consumo_medio_kml: CONSUMO_CAMINHAO_KML,
            preco_diesel_reais_litro: PRECO_DIESEL_REAIS,
        },
    };

    println!("Cálculo concluído.");
    comparacao
}


pub fn alimentar_previsao(estado: &EstadoOtimizacao, dados: DadosPrevisao) {
    println!("Atualizando previsão: {:?}", dados);
    estado.servico_demanda.lock().unwrap().atualizar_previsao(dados);
}

pub fn alimentar_distancia(estado: &EstadoOtimizacao, dados: PedidoNovaDistancia) {
    println!("Adicionando nova distância: {:?}", dados);
    estado.servico_distancia.lock().unwrap().adicionar_distancia(
        dados.origem,
        dados.destino,
        dados.custo
    );
}