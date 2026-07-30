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

<!-- Próxima entrada: Fase 1 — coletor Docker -->
