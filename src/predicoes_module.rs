
use crate::dataset::Dataset;
use crate::modelo::ModeloML;
use crate::predicao::Predicao;
use crate::ia_api;

use std::io::{self, Write};
use std::path::Path;

pub async fn run_prediction_module() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- 🔮 Módulo de Predição e Análise de IA ---");


    let tipo = read_input("Tipo de lixo (plastico, papel, vidro, metal, organico): ");
    if tipo.is_empty() {
        println!("Tipo inválido. Encerrando.");
        return Ok(());
    }

    let quantidade_str = read_input("Quantidade (kg): ");
    let quantidade: f32 = match quantidade_str.parse() {
        Ok(v) if v > 0.0 => v,
        _ => {
            println!("Quantidade inválida. Encerrando.");
            return Ok(());
        }
    };

    let observacoes_raw = read_input("Observações (opcional): ");
    let observacoes = if observacoes_raw.is_empty() {
        None
    } else {
        Some(observacoes_raw)
    };

    let db_path = Path::new("data/db.json");
    let mut dataset = Dataset::load_from_file(db_path).unwrap_or_else(|_| Dataset::new());

    dataset.add_entry(tipo.clone(), quantidade, observacoes.clone()); // Passa o clone de observacoes
    dataset.save_to_file(db_path)?;

    println!("Entrada salva com sucesso em {:?}", db_path);

    let mut modelo = ModeloML::new("ModeloSimuladoReciclagem");
    modelo.treinar(&dataset);
    let _ = modelo.salvar("output/modelo.json");

    let predicao: Predicao = modelo.prever(&dataset);
    predicao.mostrar_terminal();

    let factor = co2_factor(&tipo);
    let co2_saved = quantidade * factor;
    println!(
        "\nEstimativa imediata: reciclar {:.3} kg de {} => ~{:.3} kg CO₂ evitado (fator {:.2})",
        quantidade, tipo, co2_saved, factor
    );

    if let Some(trend) = dataset.trend_percent(&tipo, 3) {
        println!(
            "Tendência (média das últimas 3 vs anteriores 3) para {}: {:+.2}%",
            tipo, trend
        );
    } else {
        println!("Dados insuficientes para calcular tendência para '{}'.", tipo);
    }

    predicao.exportar()?;
    println!("Arquivos exportados: output/predicao.json, output/predicao.csv, Mensagens/predicao.txt");

    println!("\n🤖 Gerando análise avançada com IA...");

    let prompt = format!(
        "O usuário coletou {:.3} kg de {}. Considerando os dados históricos, forneça uma BREVE previsão ilustrativa sobre impacto ambiental e tendências. Responda em até 50 palavras.",
        quantidade, tipo
    );

    match ia_api::gerar_resposta_preditiva(&prompt).await {
        Ok(resposta) => println!("🔎 Previsão da IA: {}", resposta),
        Err(e) => eprintln!("⚠️ Erro ao gerar previsão com Gemini: {}", e),
    }

    Ok(())
}
fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    let _ = io::stdout().flush();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    buf.trim().to_string()
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