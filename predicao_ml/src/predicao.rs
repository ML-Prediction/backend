use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{create_dir_all, File};
use std::io::Write;

#[derive(Clone, Serialize, Deserialize)]
pub struct Predicao {
    pub tipos_lixo: Vec<String>,
    pub quantidades: Vec<f32>,
    pub resultados: Vec<f32>,
    pub impacto_total: f32,
    pub timestamp: String,
    pub modelo: String,
}

impl Predicao {
    pub fn new(
        tipos_lixo: Vec<String>,
        quantidades: Vec<f32>,
        resultados: Vec<f32>,
        impacto_total: f32,
        modelo: String,
    ) -> Self {
        Predicao {
            tipos_lixo,
            quantidades,
            resultados,
            impacto_total,
            timestamp: Utc::now().to_rfc3339(),
            modelo,
        }
    }

    pub fn mostrar_terminal(&self) {
        println!("=== Predição (simulada) ===");
        println!("Timestamp: {}", self.timestamp);
        println!("Modelo: {}", self.modelo);
        for (i, tipo) in self.tipos_lixo.iter().enumerate() {
            println!(
                "  - {}: {} kg -> Impacto estimado: {:.2}",
                tipo, self.quantidades[i], self.resultados[i]
            );
        }
        println!("Impacto total estimado: {:.2}", self.impacto_total);
    }

    pub fn exportar(&self) -> Result<(), Box<dyn Error>> {
        create_dir_all("output")?;
        create_dir_all("Mensagens")?;

        // JSON
        let mut f_json = File::create("output/predicao.json")?;
        let s = serde_json::to_string_pretty(&self)?;
        f_json.write_all(s.as_bytes())?;

        // CSV
        let mut f_csv = File::create("output/predicao.csv")?;
        writeln!(f_csv, "tipo_lixo,quantidade,impacto")?;
        for (i, tipo) in self.tipos_lixo.iter().enumerate() {
            writeln!(
                f_csv,
                "{},{:.3},{:.3}",
                tipo, self.quantidades[i], self.resultados[i]
            )?;
        }

        // TXT
        let mut f_txt = File::create("Mensagens/predicao.txt")?;
        writeln!(f_txt, "=== Relatório de Predição ===")?;
        writeln!(f_txt, "Timestamp: {}", self.timestamp)?;
        writeln!(f_txt, "Modelo: {}", self.modelo)?;
        for (i, tipo) in self.tipos_lixo.iter().enumerate() {
            writeln!(
                f_txt,
                "Tipo: {}, Quantidade: {:.3} kg, Impacto: {:.3}",
                tipo, self.quantidades[i], self.resultados[i]
            )?;
        }
        writeln!(f_txt, "Impacto total estimado: {:.3}", self.impacto_total)?;
        Ok(())
    }
}
