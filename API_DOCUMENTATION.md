# Documentação da API - Sistema de Gestão de Resíduos

## Base URL
```
http://localhost:8080
```

## Autenticação

A API utiliza autenticação por credenciais simples. Todos os endpoints protegidos exigem que você envie `nome` e `senha` no body da requisição (exceto os endpoints públicos).

**Nota:** Não há sistema de tokens. As credenciais devem ser enviadas em cada requisição.

---

## Estrutura de Resposta Padrão

Todas as respostas da API seguem o seguinte formato:

```typescript
interface ApiResponse<T> {
  success: boolean;
  data: T | null;      // null quando success = false
  message: string | null;  // null quando success = true
}
```

**Exemplo de sucesso:**
```json
{
  "success": true,
  "data": { ... },
  "message": null
}
```

**Exemplo de erro:**
```json
{
  "success": false,
  "data": null,
  "message": "Mensagem de erro aqui"
}
```

---

## Perfis de Usuário

A API possui 3 níveis de permissão:

- **Comum**: Pode inserir dados de coleta
- **Técnico**: Pode inserir dados, executar pré-processamento, predições e otimização de rotas
- **Administrador**: Todas as permissões + gerenciamento de usuários

---

## Endpoints

### 1. Health Check

Verifica se a API está funcionando. Não requer autenticação.

**Endpoint:** `GET /health`

**Headers:**
```
Content-Type: application/json (opcional)
```

**Response 200 (Sucesso):**
```json
{
  "success": true,
  "data": {
    "status": "ok",
    "timestamp": "2024-01-15T10:30:00Z"
  },
  "message": null
}
```

**Tipos TypeScript:**
```typescript
interface HealthResponse {
  status: string;
  timestamp: string; // ISO 8601 format
}
```

---

### 2. Login

Autentica um usuário no sistema e retorna informações do usuário.

**Endpoint:** `POST /auth/login`

**Headers:**
```
Content-Type: application/json
```

**Body:**
```json
{
  "nome": "admin",
  "senha": "admin"
}
```

**Tipos TypeScript:**
```typescript
interface LoginRequest {
  nome: string;
  senha: string;
}
```

**Response 200 (Sucesso):**
```json
{
  "success": true,
  "data": {
    "usuario": {
      "id": 1,
      "nome": "admin",
      "perfil": "Administrador"
    },
    "mensagem": "Login realizado com sucesso! Use suas credenciais nos próximos requests."
  },
  "message": null
}
```

**Tipos TypeScript:**
```typescript
interface LoginResponse {
  usuario: UsuarioResponse;
  mensagem: string;
}

interface UsuarioResponse {
  id: number;
  nome: string;
  perfil: "Comum" | "Tecnico" | "Administrador";
}
```

**Response 200 (Erro):**
```json
{
  "success": false,
  "data": null,
  "message": "Usuário não encontrado"
}
```
ou
```json
{
  "success": false,
  "data": null,
  "message": "Senha incorreta"
}
```

---

### 3. Criar Usuário

Cria um novo usuário no sistema. **Requer perfil Administrador.**

**Endpoint:** `POST /auth/usuarios`

**Headers:**
```
Content-Type: application/json
```

**Body:**
```json
{
  "nome": "admin",
  "senha": "admin",
  "novo_usuario": {
    "nome": "novo_usuario",
    "senha": "senha123",
    "perfil": "Comum"
  }
}
```

**Tipos TypeScript:**
```typescript
interface CriarUsuarioRequest {
  nome: string;           // Credenciais do admin
  senha: string;          // Credenciais do admin
  novo_usuario: {
    nome: string;
    senha: string;
    perfil: "Comum" | "Tecnico" | "Administrador";
  };
}
```

**Response 200 (Sucesso):**
```json
{
  "success": true,
  "data": {
    "id": 2,
    "nome": "novo_usuario",
    "perfil": "Comum"
  },
  "message": null
}
```

**Response 200 (Erro):**
```json
{
  "success": false,
  "data": null,
  "message": "Acesso negado. Apenas administradores podem criar usuários."
}
```
ou
```json
{
  "success": false,
  "data": null,
  "message": "Perfil inválido"
}
```

---

### 4. Inserir Dados de Coleta

Insere dados de coleta de resíduos no sistema. **Requer perfil Comum, Técnico ou Administrador.**

**Endpoint:** `POST /coleta`

**Headers:**
```
Content-Type: application/json
```

**Body:**
```json
{
  "nome": "admin",
  "senha": "admin",
  "tipo": "plastico",
  "quantidade": 150.5,
  "observacoes": "Coleta realizada na região central"
}
```

**Tipos TypeScript:**
```typescript
interface InserirColetaRequest {
  nome: string;
  senha: string;
  tipo: string;              // Ex: "plastico", "papel", "vidro", "metal", "organico"
  quantidade: number;         // Em kg (float)
  observacoes?: string;       // Opcional
}
```

**Response 200 (Sucesso):**
```json
{
  "success": true,
  "data": "Dados inseridos com sucesso",
  "message": null
}
```

**Response 200 (Erro):**
```json
{
  "success": false,
  "data": null,
  "message": "Acesso negado"
}
```

---

### 5. Listar Dados de Coleta

Lista todos os dados de coleta. Pode filtrar por tipo e usar paginação. **Não requer autenticação.**

**Endpoint:** `GET /coleta`

**Headers:**
```
Content-Type: application/json (opcional)
```

**Query Parameters:**
- `tipo` (opcional): Filtrar por tipo de resíduo
- `limit` (opcional): Número máximo de registros (padrão: 100)
- `offset` (opcional): Número de registros a pular (padrão: 0)

**Exemplo de URL:**
```
GET /coleta?tipo=plastico&limit=10&offset=0
```

**Response 200 (Sucesso):**
```json
{
  "success": true,
  "data": [
    {
      "tipo": "plastico",
      "quantidade": 150.5,
      "observacoes": "Coleta realizada na região central",
      "timestamp": "2024-01-15T10:30:00Z"
    },
    {
      "tipo": "papel",
      "quantidade": 75.0,
      "observacoes": null,
      "timestamp": "2024-01-15T11:00:00Z"
    }
  ],
  "message": null
}
```

**Tipos TypeScript:**
```typescript
interface WasteEntry {
  tipo: string;
  quantidade: number;
  observacoes: string | null;
  timestamp: string; // ISO 8601 format
}
```

---

### 6. Executar Pré-processamento

Executa o pré-processamento dos dados. **Requer perfil Técnico ou Administrador.**

**Endpoint:** `POST /preprocessamento/executar`

**Headers:**
```
Content-Type: application/json
```

**Body:**
```json
{
  "nome": "admin",
  "senha": "admin"
}
```

**Tipos TypeScript:**
```typescript
interface AuthCredentials {
  nome: string;
  senha: string;
}
```

**Response 200 (Sucesso):**
```json
{
  "success": true,
  "data": "Pré-processamento executado com sucesso",
  "message": null
}
```

**Response 200 (Erro):**
```json
{
  "success": false,
  "data": null,
  "message": "Acesso negado"
}
```

---

### 7. Executar Predição

Executa uma predição usando IA (Gemini) baseada nos dados de coleta. **Requer perfil Técnico ou Administrador.**

**Endpoint:** `POST /predicoes`

**Headers:**
```
Content-Type: application/json
```

**Body:**
```json
{
  "nome": "admin",
  "senha": "admin",
  "tipo": "plastico",
  "quantidade": 200.0,
  "observacoes": "Coleta em área residencial"
}
```

**Tipos TypeScript:**
```typescript
interface PredicaoRequest {
  nome: string;
  senha: string;
  tipo: string;
  quantidade: number;      // Em kg (float)
  observacoes?: string;
}
```

**Response 200 (Sucesso):**
```json
{
  "success": true,
  "data": {
    "predicao": {
      "tipos_lixo": ["plastico"],
      "quantidades": [200.0],
      "resultados": [450.5],
      "impacto_total": 450.5,
      "timestamp": "2024-01-15T10:30:00Z",
      "modelo": "ModeloSimuladoReciclagem"
    },
    "analise_ia": "A coleta de 200.0 kg de plástico representa uma redução significativa de emissões de CO2. A reciclagem deste material pode evitar aproximadamente 400 kg de CO2 equivalente, contribuindo para a sustentabilidade ambiental.",
    "co2_estimado": {
      "tipo": "plastico",
      "quantidade": 200.0,
      "fator": 2.0,
      "co2_evitado": 400.0
    }
  },
  "message": null
}
```

**Tipos TypeScript:**
```typescript
interface PredicaoResponse {
  predicao: {
    tipos_lixo: string[];
    quantidades: number[];
    resultados: number[];
    impacto_total: number;
    timestamp: string;      // ISO 8601 format
    modelo: string;
  };
  analise_ia: string;      // Análise gerada pela IA
  co2_estimado: {
    tipo: string;
    quantidade: number;
    fator: number;         // Fator de conversão CO2 por kg
    co2_evitado: number;    // quantidade * fator
  };
}
```

**Fatores CO2 por tipo:**
- `plastico` / `plástico`: 2.0
- `papel`: 1.2
- `vidro`: 0.6
- `metal`: 3.0
- `organico` / `orgânico`: 0.3
- Outros: 1.0

**Response 200 (Erro):**
```json
{
  "success": false,
  "data": null,
  "message": "Acesso negado"
}
```

---

### 8. Otimizar Rota

Otimiza uma rota de coleta usando algoritmos de vizinho mais próximo (guloso) e por prioridade. **Requer perfil Técnico ou Administrador.**

**Endpoint:** `POST /otimizacao/rotas`

**Headers:**
```
Content-Type: application/json
```

**Body:**
```json
{
  "nome": "admin",
  "senha": "admin",
  "pedido": {
    "garagem_id": "garagem",
    "pontos_a_visitar": [
      "ponto_A",
      "ponto_B",
      "ponto_C"
    ]
  }
}
```

**Tipos TypeScript:**
```typescript
interface OtimizarRotaRequest {
  nome: string;
  senha: string;
  pedido: {
    garagem_id: string;
    pontos_a_visitar: string[];
  };
}
```

**Response 200 (Sucesso):**
```json
{
  "success": true,
  "data": {
    "rota_gulosa": {
      "tipo_otimizacao": "Vizinho Mais Próximo",
      "sequencia_pontos": [
        "garagem",
        "ponto_A",
        "ponto_B",
        "ponto_C",
        "garagem"
      ],
      "metricas": {
        "distancia_total_km": 45.5,
        "litros_consumidos": 12.3,
        "custo_financeiro_reais": 58.5
      }
    },
    "rota_prioridade": {
      "tipo_otimizacao": "Por Prioridade",
      "sequencia_pontos": [
        "garagem",
        "ponto_C",
        "ponto_A",
        "ponto_B",
        "garagem"
      ],
      "metricas": {
        "distancia_total_km": 52.0,
        "litros_consumidos": 14.1,
        "custo_financeiro_reais": 67.0
      }
    },
    "benchmark_usado": {
      "consumo_medio_kml": 3.7,
      "preco_diesel_reais_litro": 4.75
    }
  },
  "message": null
}
```

**Tipos TypeScript:**
```typescript
interface ComparacaoOtimizacao {
  rota_gulosa: RotaDetalhada;
  rota_prioridade: RotaDetalhada;
  benchmark_usado: {
    consumo_medio_kml: number;
    preco_diesel_reais_litro: number;
  };
}

interface RotaDetalhada {
  tipo_otimizacao: string;
  sequencia_pontos: string[];
  metricas: {
    distancia_total_km: number;
    litros_consumidos: number;
    custo_financeiro_reais: number;
  };
}
```

**Response 200 (Erro):**
```json
{
  "success": false,
  "data": null,
  "message": "Acesso negado"
}
```

---

### 9. Adicionar Distância

Adiciona ou atualiza uma distância entre dois pontos. A rota reversa é automaticamente adicionada. **Requer perfil Técnico ou Administrador.**

**Endpoint:** `POST /otimizacao/distancias`

**Headers:**
```
Content-Type: application/json
```

**Body:**
```json
{
  "nome": "admin",
  "senha": "admin",
  "distancia": {
    "origem": "garagem",
    "destino": "ponto_D",
    "custo": 12.5
  }
}
```

**Tipos TypeScript:**
```typescript
interface AdicionarDistanciaRequest {
  nome: string;
  senha: string;
  distancia: {
    origem: string;
    destino: string;
    custo: number;      // Em km (float)
  };
}
```

**Response 200 (Sucesso):**
```json
{
  "success": true,
  "data": "Distância adicionada com sucesso",
  "message": null
}
```

**Response 200 (Erro):**
```json
{
  "success": false,
  "data": null,
  "message": "Acesso negado"
}
```

---

### 10. Atualizar Previsão de Demanda

Atualiza a previsão de demanda para um ponto específico. **Requer perfil Técnico ou Administrador.**

**Endpoint:** `POST /otimizacao/previsao-demanda`

**Headers:**
```
Content-Type: application/json
```

**Body:**
```json
{
  "nome": "admin",
  "senha": "admin",
  "previsao": {
    "ponto_id": "ponto_A",
    "regiao": "centro",
    "previsao_demanda": 250.5
  }
}
```

**Tipos TypeScript:**
```typescript
interface AtualizarPrevisaoRequest {
  nome: string;
  senha: string;
  previsao: {
    ponto_id: string;
    regiao: string;
    previsao_demanda: number;  // Em kg (float)
  };
}
```

**Response 200 (Sucesso):**
```json
{
  "success": true,
  "data": "Previsão atualizada com sucesso",
  "message": null
}
```

**Response 200 (Erro):**
```json
{
  "success": false,
  "data": null,
  "message": "Acesso negado"
}
```

---

### 11. Listar Usuários

Lista todos os usuários do sistema. **Requer perfil Administrador.**

**Endpoint:** `GET /usuarios`

**Headers:**
```
Content-Type: application/json
```

**Body:**
```json
{
  "nome": "admin",
  "senha": "admin"
}
```

**Nota:** Para requisições GET com body, o Postman permite, mas alguns clientes HTTP podem não suportar. Considere usar POST ou incluir credenciais em headers (se implementado no futuro).

**Tipos TypeScript:**
```typescript
interface AuthCredentials {
  nome: string;
  senha: string;
}
```

**Response 200 (Sucesso):**
```json
{
  "success": true,
  "data": [
    {
      "id": 1,
      "nome": "admin",
      "perfil": "Administrador"
    },
    {
      "id": 2,
      "nome": "usuario_comum",
      "perfil": "Comum"
    },
    {
      "id": 3,
      "nome": "tecnico",
      "perfil": "Tecnico"
    }
  ],
  "message": null
}
```

**Response 200 (Erro):**
```json
{
  "success": false,
  "data": null,
  "message": "Acesso negado"
}
```

---

### 12. Deletar Usuário

Deleta um usuário do sistema. **Requer perfil Administrador.** Não é possível deletar a si mesmo.

**Endpoint:** `DELETE /usuarios/:id`

**Headers:**
```
Content-Type: application/json
```

**URL Parameters:**
- `id` (path): ID do usuário a deletar

**Body:**
```json
{
  "nome": "admin",
  "senha": "admin"
}
```

**Exemplo de URL:**
```
DELETE /usuarios/2
```

**Tipos TypeScript:**
```typescript
interface AuthCredentials {
  nome: string;
  senha: string;
}
```

**Response 200 (Sucesso):**
```json
{
  "success": true,
  "data": "Usuário deletado com sucesso",
  "message": null
}
```

**Response 200 (Erro):**
```json
{
  "success": false,
  "data": null,
  "message": "Acesso negado"
}
```
ou
```json
{
  "success": false,
  "data": null,
  "message": "Você não pode deletar a si mesmo"
}
```
ou
```json
{
  "success": false,
  "data": null,
  "message": "Usuário não encontrado"
}
```

---

## Códigos de Status HTTP

- **200 OK**: Requisição bem-sucedida (sucesso ou erro retornado no body)
- **500 Internal Server Error**: Erro interno do servidor (raro)

**Nota:** A API retorna 200 mesmo em caso de erro de negócio. Verifique sempre o campo `success` na resposta.

---

## Exemplos de Uso em React

### Exemplo 1: Login

```typescript
const login = async (nome: string, senha: string) => {
  const response = await fetch('http://localhost:8080/auth/login', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ nome, senha }),
  });
  
  const data: ApiResponse<LoginResponse> = await response.json();
  
  if (data.success && data.data) {
    console.log('Usuário logado:', data.data.usuario);
    return data.data;
  } else {
    throw new Error(data.message || 'Erro ao fazer login');
  }
};
```

### Exemplo 2: Inserir Coleta

```typescript
const inserirColeta = async (
  nome: string,
  senha: string,
  tipo: string,
  quantidade: number,
  observacoes?: string
) => {
  const response = await fetch('http://localhost:8080/coleta', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      nome,
      senha,
      tipo,
      quantidade,
      observacoes,
    }),
  });
  
  const data: ApiResponse<string> = await response.json();
  
  if (data.success) {
    return data.data;
  } else {
    throw new Error(data.message || 'Erro ao inserir coleta');
  }
};
```

### Exemplo 3: Listar Coletas com Paginação

```typescript
const listarColetas = async (
  tipo?: string,
  limit: number = 10,
  offset: number = 0
) => {
  const params = new URLSearchParams();
  if (tipo) params.append('tipo', tipo);
  params.append('limit', limit.toString());
  params.append('offset', offset.toString());
  
  const response = await fetch(
    `http://localhost:8080/coleta?${params.toString()}`
  );
  
  const data: ApiResponse<WasteEntry[]> = await response.json();
  
  if (data.success && data.data) {
    return data.data;
  } else {
    throw new Error(data.message || 'Erro ao listar coletas');
  }
};
```

### Exemplo 4: Executar Predição

```typescript
const executarPredicao = async (
  nome: string,
  senha: string,
  tipo: string,
  quantidade: number,
  observacoes?: string
) => {
  const response = await fetch('http://localhost:8080/predicoes', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      nome,
      senha,
      tipo,
      quantidade,
      observacoes,
    }),
  });
  
  const data: ApiResponse<PredicaoResponse> = await response.json();
  
  if (data.success && data.data) {
    return data.data;
  } else {
    throw new Error(data.message || 'Erro ao executar predição');
  }
};
```

---

## Notas Importantes

1. **Autenticação**: Todas as requisições protegidas precisam incluir `nome` e `senha` no body.

2. **CORS**: A API está configurada para aceitar requisições de qualquer origem (CORS permissivo).

3. **Tipos de Resíduo**: Os tipos comuns são: `plastico`, `papel`, `vidro`, `metal`, `organico`.

4. **Formatos de Data**: Todas as datas são retornadas no formato ISO 8601 (ex: `2024-01-15T10:30:00Z`).

5. **Tratamento de Erros**: Sempre verifique o campo `success` antes de acessar `data`.

6. **GET com Body**: O endpoint `GET /usuarios` requer credenciais no body. Alguns clientes HTTP podem não suportar isso. Considere usar uma biblioteca que permita isso ou aguarde futura implementação de headers de autenticação.

---

## Versão da API

Esta documentação se refere à versão atual da API (sem versionamento explícito).

---

**Última atualização:** Janeiro 2024

