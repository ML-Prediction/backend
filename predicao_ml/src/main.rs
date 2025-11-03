mod auth;
mod actions;
mod dataset;
mod modelo;
mod predicao;
mod ia_api; // Módulo da IA (Gemini)
mod predicoes_module; // Hub da IA
pub mod otimizacao; // Módulo de Otimização (com 'pub')

use auth::{Usuario, PerfilUsuario};
use actions::{
    inserir_dados_coleta, 
    executar_pre_processamento, 
    acessar_modulo_predicoes,
    acessar_modulo_otimizacao
};
use crate::otimizacao::{EstadoOtimizacao, PedidoNovaDistancia};
use rusqlite::{Connection, Error as RusqliteError};
use std::error::Error;
use std::fs::{self, File};
use std::io::{stdin, stdout, Write};
use std::path::Path;
use bcrypt::{hash, verify, DEFAULT_COST};

const DB_FILE: &str = "sistema.db";


fn get_input(prompt: &str) -> String {
    print!("{}", prompt);
    stdout().flush().unwrap(); 
    let mut input = String::new();
    stdin().read_line(&mut input).expect("Falha ao ler entrada");
    input.trim().to_string()
}

fn init_db(conn: &Connection) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS usuarios (
            id              INTEGER PRIMARY KEY,
            nome            TEXT NOT NULL UNIQUE,
            perfil          TEXT NOT NULL,
            password_hash   TEXT NOT NULL
        )",
        [],
    )?;

    let admin_nome = "admin";
    let admin_pass = "admin";
    let admin_perfil = PerfilUsuario::Administrador.as_str();
    
    let hashed_pass = hash(admin_pass, DEFAULT_COST)?;

    conn.execute(
        "INSERT OR IGNORE INTO usuarios (nome, perfil, password_hash) VALUES (?1, ?2, ?3)",
        (admin_nome, admin_perfil, hashed_pass),
    )?;

    println!("Verificando pastas do módulo de predição...");
    fs::create_dir_all("data")?;
    fs::create_dir_all("output")?;
    fs::create_dir_all("Mensagens")?;
    
    let dist_path = "data/distancias.json";
    if !Path::new(dist_path).exists() {
        println!("Criando arquivo de distâncias padrão: {}", dist_path);
        
        let default_json_content = r#"
{
  "garagem": {
    "ponto_A": 5.2, "ponto_B": 8.1, "ponto_C": 7.5
  },
  "ponto_A": {
    "garagem": 5.2, "ponto_B": 3.0, "ponto_C": 10.8
  },
  "ponto_B": {
    "garagem": 8.1, "ponto_A": 3.0, "ponto_C": 4.4
  },
  "ponto_C": {
    "garagem": 7.5, "ponto_A": 10.8, "ponto_B": 4.4
  }
}
"#; 
        let mut file = File::create(dist_path)?;
        file.write_all(default_json_content.as_bytes())?; 
    }
    
    Ok(())
}


fn handle_create_user(conn: &Connection) -> Result<(), Box<dyn Error>> {
    println!("\n--- 📝 Criar Novo Usuário ---");
    let nome = get_input("Nome de usuário: ");
    let senha = get_input("Senha: ");
    let perfil_str = get_input("Perfil (Comum, Tecnico, Administrador): ");

    let perfil = match PerfilUsuario::try_from(perfil_str.as_str()) {
        Ok(p) => p,
        Err(e) => {
            println!("❌ Erro: {}", e);
            return Ok(());
        }
    };
    
    let password_hash = hash(senha, DEFAULT_COST)?;
    
    match conn.execute(
        "INSERT INTO usuarios (nome, perfil, password_hash) VALUES (?1, ?2, ?3)",
        (&nome, perfil.as_str(), password_hash),
    ) {
        Ok(_) => println!("✅ Usuário '{}' criado com sucesso!", nome),
        Err(e) => println!("❌ Erro ao criar usuário (talvez o nome já exista?): {}", e),
    };
    
    Ok(())
}

fn handle_login(conn: &Connection) -> Result<Option<Usuario>, Box<dyn Error>> {
    println!("\n--- 🔑 Tela de Login ---");
    let nome = get_input("Nome de usuário: ");
    let senha = get_input("Senha: ");

    let mut stmt = conn.prepare("SELECT id, nome, perfil, password_hash FROM usuarios WHERE nome = ?1")?;
    
    let login_attempt = stmt.query_row([&nome], |row| {
        let id: u32 = row.get(0)?;
        let nome_db: String = row.get(1)?;
        let perfil_str: String = row.get(2)?;
        let hash_db: String = row.get(3)?;
        
        let perfil = PerfilUsuario::try_from(perfil_str.as_str())
            .map_err(|e| RusqliteError::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(e)))?;
        
        let senha_valida = verify(senha.as_str(), &hash_db)
            .map_err(|bcrypt_err| {
                RusqliteError::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(bcrypt_err))
            })?;
        
        if senha_valida {
            Ok(Usuario { id, nome: nome_db, perfil })
        } else {
            Err(RusqliteError::InvalidQuery)
        }
    });

    match login_attempt {
        Ok(usuario) => {
            println!("✅ Login bem-sucedido! Bem-vindo(a), {}.", usuario.nome);
            Ok(Some(usuario))
        }
        Err(RusqliteError::QueryReturnedNoRows) => {
            println!("❌ Erro: Usuário '{}' não encontrado.", nome);
            Ok(None)
        }
        Err(RusqliteError::InvalidQuery) => {
            println!("❌ Erro: Senha incorreta.");
            Ok(None)
        }
        Err(e) => {
            println!("❌ Erro de banco de dados ou verificação: {}", e);
            Err(e.into())
        }
    }
}


async fn show_app_menu(
    usuario: &Usuario, 
    conn: &Connection, 
    estado_otim: &EstadoOtimizacao
) {
    println!("\n--- 🖥️  Menu Principal ---");
    println!("Logado como: {} (Perfil: {:?})", usuario.nome, usuario.perfil);
    
    loop {
        if usuario.pode_inserir_dados() {
            println!("[1] Inserir dados de coleta");
        }
        if usuario.pode_pre_processar() {
            println!("[2] Executar pré-processamento");
            println!("[3] Acessar módulo de predições (IA)");
        }
        if usuario.pode_otimizar_rotas() {
            println!("[4] Otimizar Rotas de Coleta"); 
        }
        if usuario.pode_gerenciar_usuarios() {
            println!("[5] Gerenciar usuários (Admin)"); 
        }
        if usuario.pode_otimizar_rotas() {
            println!("[6] Adicionar Distância");
        }
        println!("[0] Sair (Logout)");
        
        let escolha = get_input("Sua escolha: ");
        
        match escolha.as_str() {
            "1" if usuario.pode_inserir_dados() => {
                println!("\n--- 📥 Inserir Dados de Coleta ---");
                
                let tipo = get_input("  -> Tipo (plastico, papel, etc.): ");
                if tipo.is_empty() {
                    println!("❌ Tipo não pode ser vazio.");
                    continue;
                }

                let quantidade_str = get_input("  -> Quantidade (kg): ");
                let quantidade: f32 = match quantidade_str.parse() {
                    Ok(v) if v > 0.0 => v,
                    _ => {
                        println!("❌ Quantidade inválida. Deve ser um número maior que 0.");
                        continue;
                    }
                };

                let obs_raw = get_input("  -> Observações (opcional, pressione Enter): ");
                let observacoes = if obs_raw.is_empty() {
                    None
                } else {
                    Some(obs_raw)
                };

                if let Err(e) = inserir_dados_coleta(
                    usuario, 
                    tipo, 
                    quantidade, 
                    observacoes
                ).await {
                    println!("{}", e);
                }
            }
            "2" if usuario.pode_pre_processar() => {
                if let Err(e) = executar_pre_processamento(usuario) {
                    println!("{}", e);
                }
            }
            "3" if usuario.pode_acessar_predicoes() => {
                if let Err(e) = acessar_modulo_predicoes(usuario).await {
                    println!("{}", e);
                }
            }
            "4" if usuario.pode_otimizar_rotas() => { 
                if let Err(e) = acessar_modulo_otimizacao(usuario, estado_otim) {
                    println!("{}", e);
                }
            }
            "5" if usuario.pode_gerenciar_usuarios() => {
                if let Err(e) = handle_manage_users(usuario, conn) {
                    println!("Erro no módulo de gerenciamento: {}", e);
                }
            }
            "6" if usuario.pode_otimizar_rotas() => {
                if let Err(e) = handle_add_distancia(estado_otim) {
                    println!("Erro ao adicionar distância: {}", e);
                }
            }
            "0" => {
                println!("Fazendo logout...");
                break;
            }
            _ => println!("❌ Opção inválida ou não permitida para seu perfil."),
        }
        println!("---------------------------");
    }
}


fn handle_manage_users(admin: &Usuario, conn: &Connection) -> Result<(), Box<dyn Error>> {
    if !admin.pode_gerenciar_usuarios() {
        println!("❌ Acesso negado.");
        return Ok(());
    }
    loop {
        println!("\n--- 🛠️  Gerenciar Usuários ---");
        println!("[1] Listar todos os usuários");
        println!("[2] Deletar um usuário");
        println!("[0] Voltar ao menu principal");

        let escolha = get_input("Sua escolha: ");
        match escolha.as_str() {
            "1" => list_users(conn)?,
            "2" => delete_user(conn, admin.id)?,
            "0" => break,
            _ => println!("Opção inválida."),
        }
    }
    Ok(())
}

fn list_users(conn: &Connection) -> Result<(), Box<dyn Error>> {
    println!("\n--- 👥 Lista de Usuários ---");

    let mut stmt = conn.prepare("SELECT id, nome, perfil FROM usuarios ORDER BY id")?;
    
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, u32>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;


    for row in rows {
        let (id, nome, perfil) = row?;
        println!("  [ID: {}] - Nome: {} - Perfil: {}", id, nome, perfil);
    }
    Ok(())
}

fn handle_add_distancia(estado: &EstadoOtimizacao) -> Result<(), Box<dyn Error>> {
    println!("\n--- 🗺️  Adicionar Nova Distância ---");
    let origem = get_input("  -> Ponto de Origem: ");
    let destino = get_input("  -> Ponto de Destino: ");
    let custo_str = get_input("  -> Custo (Distância/Tempo): ");

    let custo: f64 = match custo_str.parse() {
        Ok(v) if v > 0.0 => v,
        _ => {
            println!("❌ Custo inválido. Deve ser um número maior que 0.");
            return Ok(());
        }
    };

    let dados = PedidoNovaDistancia {
        origem: origem.clone(),
        destino: destino.clone(),
        custo,
    };
    
    otimizacao::alimentar_distancia(estado, dados);

    let dados_reversos = PedidoNovaDistancia {
        origem: destino,
        destino: origem,
        custo,
    };
    otimizacao::alimentar_distancia(estado, dados_reversos);

    println!("✅ Distância (e rota reversa) adicionada/atualizada com sucesso!");
    Ok(())
}

fn delete_user(conn: &Connection, admin_id: u32) -> Result<(), Box<dyn Error>> {
    println!("\n--- ❌ Deletar Usuário ---");
    let id_str = get_input("Digite o ID do usuário a deletar: ");
    
    let id_to_delete: u32 = match id_str.parse() {
        Ok(id) => id,
        Err(_) => {
            println!("ID inválido. Deve ser um número.");
            return Ok(());
        }
    };

    if id_to_delete == admin_id {
        println!("❌ Você não pode deletar a si mesmo!");
        return Ok(());
    }

    let changes = conn.execute(
        "DELETE FROM usuarios WHERE id = ?1",
        [id_to_delete],
    )?;

    if changes == 0 {
        println!("Usuário com ID {} não encontrado.", id_to_delete);
    } else {
        println!("✅ Usuário com ID {} deletado com sucesso.", id_to_delete);
    }
    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    

    dotenv::dotenv().ok();

    let conn = Connection::open(DB_FILE)?;
    println!("Base de dados '{}' carregada.", DB_FILE);
    
    init_db(&conn)?;


    let estado_otimizacao = otimizacao::EstadoOtimizacao::new();

    loop {
        println!("\n--- Sistema de Gestão de Resíduos ---");
        println!("[1] Fazer Login");
        println!("[2] Criar Novo Usuário");
        println!("[3] Sair do Programa");
        
        let escolha = get_input("Sua escolha: ");
        
        match escolha.as_str() {
            "1" => {
                match handle_login(&conn) {
                    Ok(Some(usuario_logado)) => {
                        show_app_menu(&usuario_logado, &conn, &estado_otimizacao).await;
                    }
                    Ok(None) => continue,
                    Err(e) => println!("Erro crítico no login: {}", e),
                }
            }
            "2" => {
                if let Err(e) = handle_create_user(&conn) {
                    println!("Erro ao criar usuário: {}", e);
                }
            }
            "3" => {
                println!("Saindo...");
                break;
            }
            _ => println!("❌ Opção inválida, tente novamente."),
        }
    }
    Ok(())
} 