use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasteEntry {
    pub tipo: String,
    pub quantidade: f32,
    pub observacoes: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub entries: Vec<WasteEntry>,
}

impl Dataset {
    pub fn new() -> Self {
        Dataset { entries: Vec::new() }
    }

    pub fn add_entry(&mut self, tipo: String, quantidade: f32, observacoes: Option<String>) {
        let entry = WasteEntry {
            tipo,
            quantidade,
            observacoes,
            timestamp: Utc::now(),
        };
        self.entries.push(entry);
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        if let Some(parent) = path.as_ref().parent() {
            create_dir_all(parent)?;
        }
        let mut f = File::create(path)?;
        let s = serde_json::to_string_pretty(&self)?;
        f.write_all(s.as_bytes())?;
        Ok(())
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        if !path.as_ref().exists() {
            return Ok(Dataset::new());
        }
        let mut f = File::open(path)?;
        let mut s = String::new();
        f.read_to_string(&mut s)?;
        let ds: Dataset = serde_json::from_str(&s)?;
        Ok(ds)
    }

    /// retorna a média das últimas `n_recent` entradas do mesmo tipo (ordenadas por timestamp)
    pub fn mean_last_n_of_type(&self, tipo: &str, n_recent: usize) -> Option<f32> {
        let mut vals: Vec<f32> = self
            .entries
            .iter()
            .filter(|e| e.tipo.eq_ignore_ascii_case(tipo))
            .map(|e| e.quantidade)
            .collect();
        if vals.is_empty() {
            return None;
        }
        // ordenar por timestamp (mais velho -> mais novo)
        vals.sort_by_key(|_| 0); // não reordena; timestamps já em ordem de inserção
        let len = vals.len();
        let start = if len > n_recent { len - n_recent } else { 0 };
        let slice = &vals[start..];
        let sum: f32 = slice.iter().sum();
        Some(sum / (slice.len() as f32))
    }

    /// calcula variação percentual entre média dos últimos n e média dos n anteriores
    pub fn trend_percent(&self, tipo: &str, n: usize) -> Option<f32> {
        let vals: Vec<f32> = self
            .entries
            .iter()
            .filter(|e| e.tipo.eq_ignore_ascii_case(tipo))
            .map(|e| e.quantidade)
            .collect();
        if vals.len() < n * 2 {
            return None; // dados insuficientes
        }
        let len = vals.len();
        let recent_start = len - n;
        let recent: f32 = vals[recent_start..].iter().sum::<f32>() / (n as f32);
        let prev: f32 = vals[recent_start - n..recent_start].iter().sum::<f32>() / (n as f32);
        if prev.abs() < std::f32::EPSILON {
            return None;
        }
        Some(((recent - prev) / prev) * 100.0)
    }
}
