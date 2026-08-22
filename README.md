# 📈 Carteira de Investimentos Fullstack em Rust

Uma aplicação web completa e performática para gestão de carteira de investimentos, construída em **Rust** utilizando **Axum**, **SQLx (PostgreSQL)**, **Askama (SSR)** e integração em tempo real com a API do **Yahoo Finance**.

> 📌 **Projeto Base:** Inspirado no desafio da [Digital Innovation One](https://github.com/digitalinnovationone/rust-fullstack-carteira-investimentos), aprimorado com consulta dinâmica de cotações externas, cálculo em tempo real de ativos e ajustes de arquitetura TLS.

---

# Mapeamento Geral da Aplicação

## Arquitetura do Projeto

A aplicação é uma API/Aplicação Web Server-Side Rendered (SSR) construída em Rust, utilizando uma arquitetura modular por camadas:

* Apresentação / Roteamento (`src/routes/` & `templates/`): Utiliza Axum para manipular requisições HTTP e Askama para renderizar templates HTML direto no backend.
* Camada de Aplicação / Regra de Negócio (`src/app.rs`, `src/auth/`): Gerencia o estado da aplicação (AppState), autenticação JWT com cookies e permissões de acesso (Usuário vs. Admin).
* Integração Externa (`fetch_unit_value_in_usd`): Consulta a API pública do Yahoo Finance utilizando Reqwest para obter a cotação atualizada de ativos em tempo real em USD
* Persistência de Dados (`src/repository/`): Interage com o banco PostgreSQL através do SQLx, utilizando operações assíncronas e tipagem forte.

## Arquivos Mais Importantes

| Arquivo / Diretório     | Responsabilidade Principal                                                                                                                      |
| :---------------------- | :---------------------------------------------------------------------------------------------------------------------------------------------- |
| **`Cargo.toml`**        | Declaração de dependências (`axum`, `sqlx`, `reqwest`, `askama`, `jwt-simple`, `rust_decimal`, etc.).                                           |
| **`src/main.rs`**       | Ponto de entrada. Carrega variáveis de ambiente (`.env`), inicializa o pool do PostgreSQL (`AppState`) e inicia o servidor Axum.                |
| **`src/routes/api.rs`** | Endpoints da API REST e formulários (`/assets`, `/purchases/create`, `/purchases/{id}/delete`). Contém a integração com a API do Yahoo Finance. |
| **`src/routes/web.rs`** | Handlers das páginas renderizadas no servidor (Dashboard, Login, Telas de Ativos) via Askama.                                                   |
| **`src/repository/`**   | Camada DAO/Acesso ao banco de dados. Contém os métodos SQL assíncronos (`list_assets`, `insert_owned_asset`, `delete_purchase`, etc.).          |
| **`src/auth/`**         | Extração, geração e validação de tokens JWT via cookies assinados para controle de acesso (User vs. Admin).                                     |
| **`templates/`**        | Arquivos HTML/Askama que compõem a interface do usuário (com Tailwind CSS e gráficos Chart.js).                                                 |

# Fluxo Principal da Aplicação

1. **Autenticação:** O usuário acessa a plataforma e faz login. A aplicação gera um JWT assinado e o injeta num cookie seguro.
2. **Navegação e Leitura:** Ao acessar `/assets`, o handler recupera os ativos e compras do usuário via `Repository`, calcula o portfólio e renderiza a tela com o gráfico de distribuição de ativos.
3. **Adição de Ativo/Compra:**
   * O usuário submete o formulário de compra informando o ticker (ex: `BTC`, `EUR`, `GLD`) e a quantidade.
   * O backend intercepta e executa uma chamada HTTP para o Yahoo Finance (`fetch_unit_value_in_usd`).
   * O preço atualizado em USD é obtido, normalizado como Decimal e salvo no banco de dados (`owned_assets` / `assets`).
4. **Recálculo & Feedback:** A tela é redirecionada e atualizada instantaneamente com as cotações em tempo real e a nova composição da carteira.

---

## 🚀 Principais Melhorias Implementadas

Em relação à versão original do projeto, foram introduzidas as seguintes evoluções técnicas:

1. **Cotação Automática via Yahoo Finance:**
   - Integração assíncrona utilizando `reqwest` com suporte a TLS seguro (`rustls-tls`).
   - Normalização automática de ativos para a moeda base em **USD** (`EURUSD=X`, `BRLUSD=X`, `BTC-USD`, `ETH-USD`, `GLD`, etc.).
   
2. **Registro Dinâmico sem Entrada Manual de Preço:**
   - O preço unitário dos ativos não precisa mais ser inserido manualmente pelo usuário. Ao cadastrar um ativo ou realizar uma compra, o sistema consulta a API e armazena a cotação real atualizada.

3. **Resolução de Conflitos de Linkage & Compilação Rust:**
   - Resolução de empréstimo e tempo de vida de variáveis no compilador Rust (`Error E0716`).
   - Ajuste das dependências do `reqwest` (`default-features = false` + `rustls-tls`) para evitar colisões entre o OpenSSL nativo e a biblioteca C `boring-sys` utilizada pela dependência `jwt-simple`.

4. **Módulo de Exclusão de Aportes/Compras:**
   - Adição de endpoint para deletar registros em `owned_assets` de forma segura com validação por usuário autenticado.

---

## 🛠️ Tecnologias Utilizadas

- **Linguagem:** Rust (Edition 2024)
- **Framework Web:** [Axum v0.8](https://github.com/tokio-rs/axum)
- **Runtime Assíncrono:** [Tokio](https://tokio.rs/)
- **Banco de Dados & ORM/Query Builder:** PostgreSQL & [SQLx v0.9](https://github.com/launchbadge/sqlx)
- **Template Engine (SSR):** [Askama](https://github.com/djc/askama)
- **Precisão Financeira:** [rust_decimal](https://crates.io/crates/rust_decimal)
- **Cliente HTTP:** [Reqwest](https://github.com/seanmonstar/reqwest) (com `rustls-tls` e `json`)
- **Autenticação:** JWT (`jwt-simple`) via cookies assinados (`axum-extra`)

---

## 🏗️ Estrutura do Projeto

```text
investment-portfolio/
├── src/
│   ├── app.rs            # Estado compartilhado da aplicação (AppState)
│   ├── auth/             # Autenticação JWT e middlewares de autorização
│   ├── error.rs          # Tratamento unificado de erros (AppError)
│   ├── main.rs           # Ponto de entrada, variáveis de ambiente e pool PostgreSQL
│   ├── models.rs         # Estruturas do domínio (Asset, Purchase, User)
│   ├── repository/       # Camada DAO de acesso a banco via SQLx
│   └── routes/
│       ├── api.rs        # API REST, formulários POST e integração Yahoo Finance
│       └── web.rs        # Handlers das páginas renderizadas via Askama (SSR)
├── templates/            # Arquivos HTML/Askama com Tailwind e Chart.js
├── migrations/           # Scripts SQL do banco de dados
├── Cargo.toml            # Dependências e configurações do projeto
└── docker-compose.yml    # Configuração do PostgreSQL para ambiente local

---
# Como Executar o Projeto Localmente

## Pré-requisitos

   * Rust (versão 1.80+)

   * Docker e Docker Compose
```

1. Clonar o repositório

```text
git clone [https://github.com/rafaelribeiro-s/investment-portfolio.git](https://github.com/SEU_USUARIO/investment-portfolio.git)
cd investment-portfolio
```
2. Configurar Variáveis de Ambiente

Crie um arquivo .env na raiz do projeto com o seguinte conteúdo:

```text
DATABASE_URL=postgres://postgres:postgres@localhost:5432/portfolio
JWT_SECRET=sua_chave_secreta_super_segura

```
3.  Subir o Banco de Dados com Docker

```text
docker compose up -d
```

4. Rodar as Migrações do Banco

```text
cargo sqlx migrate run
```
5. Executar a Aplicação

```text
cargo run
```
Acesse no navegador: http://localhost:3000