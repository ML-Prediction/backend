use crate::dataset::Dataset;
use crate::predicao::Predicao;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};

#[derive(Clone, Serialize, Deserialize)]
pub struct ModeloML {
    pub tipo: String,
    pub parametros: Vec<f32>, // por exemplo [coeficiente]
    pub treinado: bool,
}

impl ModeloML {
    pub fn new(tipo: &str) -> Self {
        ModeloML {
            tipo: tipo.to_string(),
            parametros: vec![1.2], // coeficiente simulado padrão
            treinado: false,
        }
    }

    pub fn treinar(&mut self, _dataset: &Dataset) {
        // Simulação: "ajusta" o coeficiente com base em média das quantidades
        self.treinado = true;
        println!("Modelo '{}' treinado (simulado).", self.tipo);
    }

    pub fn prever(&self, dataset: &Dataset) -> Predicao {
        let coef = self.parametros.get(0).cloned().unwrap_or(1.0);
        let resultados: Vec<f32> = dataset
            .entries
            .iter()
            .map(|e| e.quantidade * coef)
            .collect();
        let impacto_total: f32 = resultados.iter().sum();
        Predicao::new(
            dataset
                .entries
                .iter()
                .map(|e| e.tipo.clone())
                .collect(),
            dataset.entries.iter().map(|e| e.quantidade).collect(),
            resultados,
            impacto_total,
            self.tipo.clone(),
        )
    }

    pub fn salvar(&self, caminho: &str) -> Result<(), Box<dyn Error>> {
        let mut f = File::create(caminho)?;
        let s = serde_json::to_string_pretty(&self)?;
        f.write_all(s.as_bytes())?;
        Ok(())
    }

    pub fn carregar(caminho: &str) -> Result<ModeloML, Box<dyn Error>> {
        let mut f = File::open(caminho)?;
        let mut s = String::new();
        f.read_to_string(&mut s)?;
        let m: ModeloML = serde_json::from_str(&s)?;
        Ok(m)
    }
}
