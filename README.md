# Carteira de Investimentos Fullstack em Rust

Uma aplicação web completa e performática para gestão de carteira de investimentos, construída em **Rust** utilizando **Axum**, **SQLx (PostgreSQL)**, **Askama (SSR)** e integração em tempo real com a API do **Yahoo Finance**.

> 📌 **Projeto Base:** Inspirado no desafio da [Digital Innovation One](https://github.com/digitalinnovationone/rust-fullstack-carteira-investimentos), aprimorado com consulta dinâmica de cotações externas, cálculo em tempo real de ativos e ajustes de arquitetura TLS.

---

## Principais Melhorias Implementadas

Em relação à versão original do projeto, foram introduzidas as seguintes evoluções técnicas:

1. **Cotação Automática via Yahoo Finance:**
   - Integração assíncrona utilizando `reqwest` com suporte a TLS seguro (`rustls-tls`).   
   
2. **Normalização automática de ativos para a moeda base em USD:**
	- `EURUSD=X`, `BRLUSD=X`, `BTC-USD`, `ETH-USD`, `GLD`, etc.

3. **Registro Dinâmico sem Entrada Manual de Preço:**
   - O preço unitário dos ativos não precisa mais ser inserido manualmente pelo usuário. Ao cadastrar um ativo ou realizar uma compra, o sistema consulta a API e armazena a cotação real atualizada.

4. **Módulo de Exclusão de Aportes/Compras:**
   - Adição de endpoint para deletar registros em `owned_assets` de forma segura com validação por usuário autenticado.

---

## Tecnologias Utilizadas

- **Linguagem:** Rust (Edition 2024)
- **Framework Web:** [Axum v0.8](https://github.com/tokio-rs/axum)
- **Runtime Assíncrono:** [Tokio](https://tokio.rs/)
- **Banco de Dados & ORM/Query Builder:** PostgreSQL & [SQLx v0.9](https://github.com/launchbadge/sqlx)
- **Template Engine (SSR):** [Askama](https://github.com/djc/askama)
- **Precisão Financeira:** [rust_decimal](https://crates.io/crates/rust_decimal)
- **Cliente HTTP:** [Reqwest](https://github.com/seanmonstar/reqwest) (com `rustls-tls` e `json`)
- **Autenticação:** JWT (`jwt-simple`) via cookies assinados (`axum-extra`)

---

## Estrutura do Projeto

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