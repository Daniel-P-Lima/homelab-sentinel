# Homelab Sentinel

Dashboard unificado para o homelab (Docker, Pi-hole, e futuramente
Uptime Kuma/Beszel), construído em Rust (axum) + Vue 3 + TypeScript.

- Documentação completa e roadmap: pasta [`docs/`](./docs) (pensada para
  abrir como vault do Obsidian).
- Comece por [`docs/00-Overview.md`](./docs/00-Overview.md).
- Setup e deploy: [`docs/04-Setup.md`](./docs/04-Setup.md).

Status atual: **Fases 0 (scaffold), 0.5 (Git + CI) e 1 (coletor Docker)
concluídas** — dashboard já lista os containers do homelab via
`GET /api/containers`, com auto-refresh no frontend.
Próximo passo: Fase 2 — integração com o Pi-hole. Ver
[`docs/02-Roadmap.md`](./docs/02-Roadmap.md) e
[`docs/03-Changelog.md`](./docs/03-Changelog.md) para o histórico completo.
