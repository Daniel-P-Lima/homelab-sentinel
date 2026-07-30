---
tags: [homelab, roadmap]
status: em-andamento
atualizado: 2026-07-30
---

# Roadmap — Homelab Sentinel

Cada fase deve terminar com algo **rodando de ponta a ponta** (backend →
frontend → visível no navegador), nunca só "meio implementado". Marcar `[x]`
só quando isso for verdade.

## Fase 0 — Scaffold ✅
- [x] Estrutura de pastas (`backend/`, `frontend/`, `docs/`)
- [x] Backend axum básico servindo `/api/health`
- [x] Frontend Vite + Vue 3 + TS básico consumindo `/api/health`
- [x] Docs iniciais no Obsidian

## Fase 0.5 — Versionamento e CI/CD
- [ ] `git init` + primeiro commit (scaffold da Fase 0)
- [ ] Repositório no GitHub (público ou privado — decidir)
- [ ] GitHub Actions: workflow de CI rodando em todo push/PR
  - `cargo build` + `cargo test` + `cargo clippy` (backend)
  - `npm run build` + `vue-tsc` (frontend, já pega erro de tipo)
- [ ] Branch protection na `main` exigindo CI verde antes de merge
- [ ] (Depois da Fase 6) Workflow de CD: build das imagens Docker e
  deploy no homelab via SSH action, disparado por tag/push na `main`

## Fase 1 — Coletor Docker (MVP real)
- [ ] `docker.rs`: listar containers via bollard (nome, status, CPU%, mem)
- [ ] Endpoint `GET /api/containers`
- [ ] `ContainerTable.vue`: tabela com auto-refresh (poll a cada 5s)
- [ ] Testar rodando de fato contra o Docker do homelab (`192.168.15.32`)

## Fase 2 — Integração Pi-hole
- [ ] Cliente HTTP para Pi-hole API (queries bloqueadas, top domínios)
- [ ] Endpoint `GET /api/pihole/summary`
- [ ] Card no dashboard com o resumo

## Fase 3 — Background jobs + persistência histórica
- [ ] Job periódico (tokio `interval`) que salva snapshots no SQLite
- [ ] Endpoint de histórico (`GET /api/containers/history?range=24h`)
- [ ] Gráfico simples no frontend (uso de CPU/mem ao longo do tempo)
- [ ] Trocar poll por SSE nesta fase

## Fase 4 — Análise de logs / detecção de anomalia
- [ ] Coletar logs de containers (bollard `logs` stream)
- [ ] Análise simples: contagem de erros/warnings, detecção de spike
- [ ] Endpoint + painel de "eventos suspeitos"

## Fase 5 — Notificações
- [ ] Webhook Telegram/Discord configurável via UI (modal + auto-save)
- [ ] Regras de alerta (threshold de CPU, container down, spike de erro)

## Fase 6 — Deploy definitivo
- [ ] Dockerfile multi-stage (build Rust + build Vue, imagem final enxuta)
- [ ] Entrada no `docker-compose.yml` do homelab (porta 8087 sugerida)
- [ ] Documentar processo de deploy em [[04-Setup]]
- [ ] Ativar o workflow de CD definido na Fase 0.5 (deploy automático
  via GitHub Actions ao dar push na `main`, em vez de `scp` manual)

---

## Como usar este roadmap

- Ao terminar uma fase, criar uma entrada em [[03-Changelog]] com data e o
  que mudou.
- Se uma fase virar duas conversas, ok — mas cada conversa deve fechar com
  o checklist atualizado.
- Não pular fase por tédio: o valor do projeto (e do artigo que o inspirou)
  está em manter o incremento pequeno e testável.
