// src/gemini_api.rs
use reqwest::{Client, header::{HeaderMap, HeaderValue, CONTENT_TYPE}};
use serde_json::json;
use std::env;

/// Função que envia um prompt de texto para a API Gemini e retorna a resposta.
/// 
/// # Exemplo de uso:
/// ```rust,ignore
/// let resposta = gemini_api::gerar_resposta_preditiva("Qual é a previsão para hoje?").await?;
/// println!("{}", resposta);
/// ```
pub async fn gerar_resposta_preditiva(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Lê a variável de ambiente GEMINI_API_KEY
    let api_key = env::var("GEMINI_API_KEY")
        .expect("A variável de ambiente GEMINI_API_KEY não está definida!");

    // Endpoint da API Gemini
    let url = "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent";

    // Monta o corpo JSON da requisição
    let body = json!({
        "contents": [
            {
                "parts": [
                    { "text": prompt }
                ]
            }
        ]
    });

    // Cabeçalhos
    let mut headers = HeaderMap::new();
    headers.insert("x-goog-api-key", HeaderValue::from_str(&api_key)?);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    // Cliente HTTP
    let client = Client::new();
    let response = client
        .post(url)
        .headers(headers)
        .json(&body)
        .send()
        .await?;

    // Converte a resposta em JSON
    let response_json: serde_json::Value = response.json().await?;

    // Extrai o texto retornado pelo modelo
    let texto_resposta = response_json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("Erro: não foi possível interpretar a resposta do modelo.")
        .to_string();

    Ok(texto_resposta)
}
