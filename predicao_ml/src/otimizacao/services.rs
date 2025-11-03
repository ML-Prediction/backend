use crate::otimizacao::models::{DadosPrevisao, PedidoOtimizacao, ResultadoRotaInterna};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File}; 
use std::io::Write;
use std::error::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ServicoDistancia { 
    matriz_custos: HashMap<String, HashMap<String, f64>>,
    
    #[serde(skip)]
    caminho_arquivo: String,
}

impl ServicoDistancia {
    
    pub fn new(caminho_json: &str) -> Self {
        let matriz_custos = match fs::read_to_string(caminho_json) {
            Ok(conteudo) => {
                serde_json::from_str(&conteudo).unwrap_or_else(|e| {
                    eprintln!("Aviso: Erro ao parsear distancias.json ({}). Começando com mapa vazio.", e);
                    HashMap::new()
                })
            },
            Err(e) => {
                eprintln!("Aviso: {} não encontrado ({}). Começando com mapa vazio.", caminho_json, e);
                HashMap::new()
            }
        };

        Self {
            matriz_custos,
            caminho_arquivo: caminho_json.to_string(),
        }
    }
    
    pub fn get_custo(&self, origem: &str, destino: &str) -> Option<f64> {
        self.matriz_custos.get(origem)?.get(destino).copied()
    }

    pub fn adicionar_distancia(&mut self, origem: String, destino: String, custo: f64) { // <-- 'pub(crate)' MUDOU PARA 'pub'
        self.matriz_custos
            .entry(origem.clone())
            .or_default()
            .insert(destino.clone(), custo);
        self.matriz_custos
            .entry(destino)
            .or_default()
            .insert(origem, custo);
            
        if let Err(e) = self.salvar() {
            eprintln!("ERRO CRÍTICO: Não foi possível salvar distancias.json: {}", e);
        }
    }

    fn salvar(&self) -> Result<(), Box<dyn Error>> {
        let conteudo_json = serde_json::to_string_pretty(&self.matriz_custos)?;
        let mut file = File::create(&self.caminho_arquivo)?;
        file.write_all(conteudo_json.as_bytes())?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ServicoDemanda {
    previsoes: HashMap<String, (String, f64)>,
}
impl ServicoDemanda {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn atualizar_previsao(&mut self, dados: DadosPrevisao) {
        self.previsoes.insert(dados.ponto_id, (dados.regiao, dados.previsao_demanda));
    }
    pub fn get_demanda_ponto(&self, ponto_id: &str) -> Option<f64> {
        self.previsoes.get(ponto_id).map(|(_regiao, demanda)| *demanda)
    }
}

pub fn otimizar_rota_vizinho_proximo(
    pedido: &PedidoOtimizacao,
    servico_distancia: &ServicoDistancia,
) -> ResultadoRotaInterna {
    let mut sequencia_pontos = Vec::new();
    let mut distancia_total_km = 0.0;
    let mut nao_visitados: HashSet<String> =
        pedido.pontos_a_visitar.iter().cloned().collect();
    let mut ponto_atual = pedido.garagem_id.clone();
    sequencia_pontos.push(ponto_atual.clone());

    while !nao_visitados.is_empty() {
        let mut vizinho_mais_proximo: Option<String> = None;
        let mut menor_custo = f64::MAX;

        for ponto_destino in nao_visitados.iter() {
            if let Some(custo) = servico_distancia.get_custo(&ponto_atual, ponto_destino) {
                if custo < menor_custo {
                    menor_custo = custo;
                    vizinho_mais_proximo = Some(ponto_destino.to_string());
                }
            }
        }
        match vizinho_mais_proximo {
            Some(proximo) => {
                distancia_total_km += menor_custo;
                ponto_atual = proximo.clone();
                sequencia_pontos.push(ponto_atual.clone());
                nao_visitados.remove(&ponto_atual);
            }
            None => break,
        }
    }
    if let Some(custo_final) = servico_distancia.get_custo(&ponto_atual, &pedido.garagem_id) {
        distancia_total_km += custo_final;
        sequencia_pontos.push(pedido.garagem_id.clone());
    }
    
    ResultadoRotaInterna { sequencia_pontos, distancia_total_km }
}

pub fn otimizar_rota_por_prioridade(
    pedido: &PedidoOtimizacao,
    servico_distancia: &ServicoDistancia,
    servico_demanda: &ServicoDemanda,
) -> ResultadoRotaInterna {
    let mut sequencia_pontos = Vec::new();
    let mut distancia_total_km = 0.0;
    let mut nao_visitados: HashSet<String> =
        pedido.pontos_a_visitar.iter().cloned().collect();
    let mut ponto_atual = pedido.garagem_id.clone();
    sequencia_pontos.push(ponto_atual.clone());

    while !nao_visitados.is_empty() {
        let mut melhor_vizinho: Option<String> = None;
        let mut maior_prioridade = f64::MIN; 

        for ponto_destino in nao_visitados.iter() {
            let distancia = servico_distancia
                .get_custo(&ponto_atual, ponto_destino)
                .unwrap_or(f64::MAX); 
            if distancia == f64::MAX { continue; }

            let demanda = servico_demanda
                .get_demanda_ponto(ponto_destino)
                .unwrap_or(0.0);
            
            let prioridade = (demanda * 1000.0) - distancia; 

            if prioridade > maior_prioridade {
                maior_prioridade = prioridade;
                melhor_vizinho = Some(ponto_destino.to_string());
            }
        }

        match melhor_vizinho {
            Some(proximo) => {
                let custo_real_da_viagem = servico_distancia
                    .get_custo(&ponto_atual, &proximo)
                    .unwrap_or(0.0);
                
                distancia_total_km += custo_real_da_viagem;
                ponto_atual = proximo.clone();
                sequencia_pontos.push(ponto_atual.clone());
                nao_visitados.remove(&ponto_atual);
            }
            None => break,
        }
    }
    if let Some(custo_final) = servico_distancia.get_custo(&ponto_atual, &pedido.garagem_id) {
        distancia_total_km += custo_final;
        sequencia_pontos.push(pedido.garagem_id.clone());
    }

    ResultadoRotaInterna { sequencia_pontos, distancia_total_km }
}