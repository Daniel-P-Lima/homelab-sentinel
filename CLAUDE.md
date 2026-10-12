# CLAUDE.md — Homelab Sentinel

Instruções para o Claude Code neste repositório. O repo fica em
`/home/mr_robot/homelab-sentinel` no servidor (`192.168.15.108`); as outras
stacks do homelab ficam em `/home/mr_robot/docker/<stack>/` (pihole, media).

---

## O que é este projeto

Dashboard unificado do homelab: agrega Docker, Pi-hole e (futuramente) Uptime
Kuma e Beszel numa tela só, em vez de cinco abas. Backend Rust + frontend Vue.

Dois objetivos, e o segundo importa tanto quanto o primeiro:
1. ter o dashboard;
2. **aprender a stack Rust/axum + Vue 3/TS na prática**, com incremento pequeno
   e testável.

Isso significa: **prefira a solução que ensina à solução que só resolve.** Não
troque uma implementação por uma crate mágica sem explicar o que ela faz.
E não amplie o escopo — o que está fora está declarado em `docs/00-Overview.md`
(não reimplementar health-check, sem multiusuário, sem app nativo).

---

## Leia os docs antes de codar

`docs/` é um vault Obsidian e é a fonte da verdade do projeto — mais atual que
qualquer suposição a partir do código:

| Arquivo | Para quê |
|---------|----------|
| `docs/00-Overview.md` | o que é, escopo e o que está **fora** de escopo |
| `docs/01-Architecture.md` | decisões técnicas e o **porquê** de cada uma |
| `docs/02-Roadmap.md` | as fases — **é aqui que se decide o que fazer agora** |
| `docs/03-Changelog.md` | histórico sessão a sessão, com as pendências reais |
| `docs/04-Setup.md` | dev, build e deploy |

Links entre docs usam sintaxe de wikilink do Obsidian (`[[01-Architecture]]`).
Mantenha o padrão ao criar referências novas.

---

## Estado atual

- **Fase 0** (scaffold), **0.5** (Git + CI) e **1** (coletor Docker) feitas.
- **Próxima: Fase 2 — integração com o Pi-hole** (cliente HTTP, endpoint
  `GET /api/pihole/summary`, card no dashboard).
- Fase 1 mergeada na `main` (PRs #1–#3). Correções do coletor (CPU% real,
  stats em paralelo, ping no boot) e deploy via `docker-compose.yml` em
  `fix/coletor-docker-e-deploy` — ver `docs/03-Changelog.md`.

---

## Estrutura

```
backend/
  Cargo.toml   Dockerfile
  src/
    main.rs      # bootstrap: router axum, layers, tokio tasks
    state.rs     # AppState (Arc<AppStateInner>), clientes externos
    routes.rs    # handlers HTTP + AppError (IntoResponse)
    docker.rs    # coletor via bollard
frontend/
  src/
    main.ts            App.vue
    api.ts             # client HTTP tipado — espelha os tipos do backend
    components/        # ContainerTable.vue
    composables/       # usePolling (genérico), useContainers, useHealth
  nginx.conf           # produção: serve o dist + proxy /api → sentinel-backend:8087
docs/                  # vault Obsidian (ver acima)
docker-compose.yml     # stack completa: docker-proxy + backend + frontend
.github/workflows/ci.yml
```

---

## Convenções

**Idioma.** Documentação, comentários e mensagens de erro voltadas ao usuário em
**português (pt-BR)**. Nomes de símbolos (structs, funções, variáveis) em inglês,
como já está no código. Comentário só onde explica *por quê*, não *o quê* — siga
a densidade do código existente.

**Backend (Rust)**
- Erros com `thiserror`. Cada módulo de coleta tem seu enum de erro (ver
  `DockerError` em `docker.rs`), que vira uma variante de `AppError`
  (`routes.rs`) via `#[from]`. Handlers devolvem `Result<Json<T>, AppError>` e
  usam `?`; o `IntoResponse` de `AppError` decide o status e monta o
  `{"error": "..."}`.
- **Dependência externa indisponível nunca derruba o processo.** O padrão já
  estabelecido: `AppState.docker` é `Option<Docker>`; o boot faz `ping` (criar o
  cliente bollard não abre conexão nenhuma), se falha loga `warn!` e segue, e o
  handler devolve `AppError::Unavailable` → **503**. Repita isso para Pi-hole,
  Uptime Kuma e o que vier.
  Nada de `.unwrap()` em handler.
- Logging com `tracing` (`info!`/`warn!`/`error!`), nunca `println!`.
- Toda mudança de schema SQLite via `sqlx migrate` — nunca `ALTER TABLE` na mão.

**Frontend (Vue 3 + TS)**
- `<script setup lang="ts">`, Composition API.
- Lógica de dados em **composables** (`useContainers()`), não solta dentro do
  `.vue`. Poll novo = `usePolling(fetchX)` — ele já evita requests sobrepostas
  e mantém o último dado bom quando uma chamada falha.
- Todo endpoint novo ganha sua função e seus tipos em `src/api.ts`. O tipo
  precisa bater com o `#[derive(Serialize)]` do backend, incluindo os
  `Option<T>` → `T | null`.
- `tsconfig` em `strict` com `noUnusedLocals` e `noUnusedParameters`: variável
  sobrando **quebra o CI**.
- Poll de 5s é o padrão do MVP. SSE só na Fase 3 — não antecipe.

---

## Comandos

**Este repositório costuma ser aberto direto no servidor do homelab, que não tem
`cargo` nem `npm` instalados.** Confira antes de rodar qualquer coisa; se não
houver toolchain, valide via Docker ou deixe para a máquina de dev.

```bash
# dev
cd backend  && cargo run                    # :8087
cd frontend && npm install && npm run dev   # :5173, proxy /api → 8087

# o que o CI vai cobrar (rode antes de commitar)
cd backend  && cargo build && cargo test && cargo clippy --all-targets -- -D warnings
cd frontend && npx vue-tsc -b && npm run build

# sem toolchain no servidor: CI do backend num container (mesma versão do Dockerfile)
docker run --rm -v "$PWD/backend":/src -w /src rust:1.80-slim \
  sh -c 'rustup component add clippy && cargo test --locked && cargo clippy --locked --all-targets -- -D warnings'
```

Deploy no homelab (`192.168.15.108`), a partir da raiz deste repo
(o usuário está no grupo `docker`, não precisa de `sudo`):

```bash
docker compose up -d --build
docker compose logs -f sentinel-backend
# acesso: http://192.168.15.108:8087
```

---

## Regra de ouro do roadmap

Cada fase termina com algo **rodando ponta a ponta** — backend → frontend →
visível no navegador. Nunca "meio implementado". Só marque `[x]` quando isso for
verdade, e não pule fase por tédio: o valor do projeto está no incremento
pequeno e testável.

Ao fechar uma fase:
1. marcar o checkbox em `docs/02-Roadmap.md`;
2. escrever a entrada em `docs/03-Changelog.md` com a data e — importante — as
   **pendências reais**. O changelog atual registra honestamente o que não deu
   pra validar ("este sandbox não tem Docker, então o fluxo completo só será
   validado no homelab"). **Mantenha esse padrão: não declare como validado o
   que você não viu funcionar.**
3. atualizar `docs/01-Architecture.md` se alguma decisão mudou.

---

## Git e CI

- Remote: `git@github.com:Daniel-P-Lima/homelab-sentinel.git`.
- Uma branch por fase (`feature/fase-2-pihole`), PR pra `main`.
- `main` tem branch protection: CI verde é obrigatório antes do merge.
- CI (`.github/workflows/ci.yml`) roda em todo push/PR pra `main`:
  - backend: `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`
  - frontend: `npm ci`, `vue-tsc -b`, `npm run build`
- **`clippy -D warnings` significa que um warning reprova o build.** Entregue
  limpo, não deixe pro CI descobrir.
- `npm ci` exige `package-lock.json` em dia — se mexer em dependência do
  frontend, commite o lock junto.
- Commit e push **só quando o usuário pedir**.

---

## Armadilhas conhecidas

1. **Pins no `Cargo.toml`.** `indexmap`, `time` e `idna_adapter` estão fixados
   com `=` porque um ambiente de dev anterior tinha Rust 1.75. Produção e CI
   usam Rust 1.80 (o CI é fixado na mesma versão do `Dockerfile` de propósito).
   Entenda esse histórico antes de despinar; e como tudo builda com `--locked`,
   o `Cargo.lock` precisa acompanhar qualquer mexida em dependência.
2. **O backend não monta o `docker.sock`.** Ele fala com o
   `tecnativa/docker-socket-proxy` via `DOCKER_HOST=tcp://docker-proxy:2375`, e
   o proxy só libera `GET` em `/containers`. Coletor novo que precise de outro
   endpoint do Docker (ex: `/events`) exige liberar a variável correspondente
   no proxy. Sem `DOCKER_HOST`, cai no socket local — é o caso do `cargo run`.
3. Fora de um ambiente com Docker o backend sobe normalmente, só que
   `/api/containers` devolve 503 — comportamento esperado, não bug.
4. **Stats levam ~1-2s por container** (`one_shot: false`, necessário pra CPU%
   ser o uso atual). Por isso são buscados em paralelo (`STATS_CONCURRENCY`).
   Não volte pra `one_shot: true` pra "acelerar".
