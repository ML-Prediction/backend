use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum PerfilUsuario {
    Comum,
    Tecnico,
    Administrador,
}

#[derive(Debug)]
pub struct PerfilParseError(String);

impl fmt::Display for PerfilParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for PerfilParseError {}

impl PerfilUsuario {
    pub fn as_str(&self) -> &'static str {
        match self {
            PerfilUsuario::Comum => "Comum",
            PerfilUsuario::Tecnico => "Tecnico",
            PerfilUsuario::Administrador => "Administrador",
        }
    }
}

impl TryFrom<&str> for PerfilUsuario {
    type Error = PerfilParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Comum" => Ok(PerfilUsuario::Comum),
            "Tecnico" => Ok(PerfilUsuario::Tecnico),
            "Administrador" => Ok(PerfilUsuario::Administrador),
            _ => Err(PerfilParseError(format!("Perfil desconhecido: '{}'", value))),
        }
    }
}

pub struct Usuario {
    pub id: u32,
    pub nome: String,
    pub perfil: PerfilUsuario,
}

impl Usuario {
    
    pub fn pode_inserir_dados(&self) -> bool {
        match self.perfil {
            PerfilUsuario::Comum | PerfilUsuario::Tecnico | PerfilUsuario::Administrador => true,
        }
    }

    pub fn pode_pre_processar(&self) -> bool {
        match self.perfil {
            PerfilUsuario::Tecnico | PerfilUsuario::Administrador => true,
            PerfilUsuario::Comum => false,
        }
    }

    pub fn pode_acessar_predicoes(&self) -> bool {
        self.pode_pre_processar()
    }

    pub fn pode_otimizar_rotas(&self) -> bool {
        match self.perfil {
            PerfilUsuario::Tecnico | PerfilUsuario::Administrador => true,
            _ => false,
        }
    }
    
    pub fn pode_gerenciar_usuarios(&self) -> bool {
        match self.perfil {
            PerfilUsuario::Administrador => true,
            _ => false,
        }
    }
}