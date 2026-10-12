---
tags: [homelab, changelog]
---

# Changelog — Homelab Sentinel

## 2026-07-30 — Fase 0: Scaffold inicial
- Criada estrutura de pastas (`backend/`, `frontend/`, `docs/`).
- Backend: axum + tokio, endpoint `GET /api/health`.
- Frontend: Vite + Vue 3 + TS, consome `/api/health` e exibe status.
- Docs iniciais: Overview, Architecture, Roadmap, Changelog.
- Decisão: poll simples no MVP, SSE só na Fase 3.
- Decisão: SQLite em vez de Postgres (ver [[01-Architecture]]).

## 2026-07-30 — Fase 1: Coletor Docker
- `backend/src/docker.rs`: conecta no Docker via socket local (bollard),
  lista todos os containers (`all: true`, inclui parados), e para os que
  estão `running` busca uma amostra de stats (CPU%, memória) com o mesmo
  cálculo do `docker stats` (delta de uso / delta do sistema × nº CPUs).
- Endpoint `GET /api/containers`: 500 com mensagem clara se o Docker
  falhar (testado neste sandbox sem Docker disponível — não crasha,
  retorna JSON de erro legível).
- `ContainerTable.vue`: tabela com nome, imagem, estado (badge
  running/exited), CPU%, memória (usage/limit formatado em MB/GB).
  Poll a cada 5s, mesmo padrão do `App.vue`.
- Build validado: `cargo build` (backend) e `vue-tsc` + `npm run build`
  (frontend) rodando limpos neste ambiente.
- **Pendência real:** este sandbox não tem Docker instalado, então o
  fluxo completo (containers de verdade aparecendo na tabela) só será
  validado no homelab. Ver [[02-Roadmap]].
- Nota técnica: tive que pinar `indexmap`, `time` e `idna_adapter` no
  `Cargo.toml` porque o Rust disponível neste sandbox (1.75, via apt) é
  mais antigo que o normal e algumas deps transitivas exigem toolchain
  mais novo. Isso não afeta o build de produção (`rust:1.80-slim` no
  Dockerfile), mas os pins ficaram no projeto por segurança/reprodutibilidade.

## 2026-07-30 — Fase 0.5: Versionamento e CI/CD
- Adicionado `.github/workflows/ci.yml`: build + test + clippy (backend),
  vue-tsc + build (frontend), rodando em todo push/PR.
- Repositório inicializado com `git init` e primeiro commit.
- Documentado processo de conexão com GitHub e branch protection em
  [[04-Setup]].
- Atualizado [[01-Architecture]] com seção de versionamento/CI/CD.
- Decisão: CD (deploy automático) só entra na Fase 6, quando o
  Dockerfile multi-stage estiver definitivo — antes disso, deploy
  manual via `scp` continua sendo mais simples que manter uma pipeline
  de deploy pra um alvo que ainda muda de forma.

## 2026-07-30 — Instruções para o agente + limpeza de build
- Adicionado `CLAUDE.md` na raiz do repo (convenções, roadmap, CI,
  armadilhas) e outro em `/home/mr_robot/homelab` (contexto de infra).
- `frontend/tsconfig.tsbuildinfo` removido do controle de versão e
  adicionado ao `.gitignore` — é artefato incremental do `vue-tsc` e
  aparecia no diff de todo PR.
- `frontend/Dockerfile`: `npm install` → `npm ci`, com o
  `package-lock.json` copiado junto. O build de produção passa a usar
  exatamente as versões que o CI validou, em vez de re-resolver.
- Adicionado `frontend/.dockerignore`: sem ele o `COPY . .` viria por
  cima do `node_modules` instalado pelo `npm ci`.

## 2026-10-07 — Correções do coletor Docker + primeiro deploy no homelab
- **CPU% era a média da vida do container, não o uso atual.** Com
  `one_shot: true` o daemon devolve `precpu_stats` zerado, e o delta
  virava o acumulado desde o start. Agora `one_shot: false`, e o cálculo
  devolve `None` se não houver amostra anterior.
- Com `one_shot: false` cada amostra leva ~1-2s, então os stats passaram
  a ser buscados em paralelo (`buffer_unordered`, até 8 por vez). Com 4
  containers, `/api/containers` responde em ~2s.
- Memória agora desconta `inactive_file` (cgroup v2) / `total_inactive_file`
  (v1), igual ao `docker stats`.
- Testes unitários para o cálculo de CPU e memória (`cargo test` rodava
  zero testes).
- **503 nunca acontecia:** criar o cliente bollard não abre conexão, então
  "Docker indisponível" só aparecia na request, como 500. Agora o boot faz
  `ping` (5 tentativas, 2s entre elas, para o caso do proxy subir depois)
  e o handler devolve 503.
- `AppError` com `IntoResponse` em `routes.rs`, como a arquitetura já
  descrevia.
- CORS removido (o navegador sempre fala com a mesma origem).
- Segurança: o backend não monta mais o `docker.sock`; fala com o
  `tecnativa/docker-socket-proxy`, que só libera `GET` em `/containers`.
  Validado: `POST /containers/pihole/stop` via proxy → 403.
- Nginx resolve `sentinel-backend` por request (`resolver 127.0.0.11`), então
  recriar o backend não quebra mais o proxy do frontend.
- Frontend: composable `usePolling` (`useContainers`, `useHealth`), que não
  empilha requests se o backend demorar e mantém a última tabela boa
  quando um poll falha.
- CI fixado em Rust 1.80, a mesma versão do `Dockerfile`, e com `--locked`.
- `docker-compose.snippet.yml` virou o `docker-compose.yml` da raiz, que
  roda sozinho. Docs atualizados com o servidor real (`192.168.15.108`,
  stacks em `~/docker/`, sem `~/homelab`).
- **Validado no homelab:** a stack subiu com `docker compose up -d --build`;
  `/api/containers` lista os 4 containers do servidor com CPU% e memória
  batendo com o `docker stats`; boot com o proxy atrasado testado. Isso
  fecha a pendência da Fase 1.
- **Pendência:** o acesso de outra máquina da LAN foi testado só pelo IP
  do próprio servidor (`curl http://192.168.15.108:8087`) — falta confirmar
  num navegador de outra máquina. `ufw` não foi verificado (precisa de
  `sudo`), mas portas publicadas pelo Docker passam por fora do `ufw`.

<!-- Próxima entrada: Fase 2 — integração Pi-hole -->
