# Homelab Sentinel

Dashboard unificado para o homelab (Docker, Pi-hole, e futuramente
Uptime Kuma/Beszel), construído em Rust (axum) + Vue 3 + TypeScript.

- Documentação completa e roadmap: pasta [`docs/`](./docs) (pensada para
  abrir como vault do Obsidian).
- Comece por [`docs/00-Overview.md`](./docs/00-Overview.md).
- Setup e deploy: [`docs/04-Setup.md`](./docs/04-Setup.md).

Status atual: **Fases 0 (scaffold), 0.5 (Git + CI), 1 (coletor Docker) e 2 (Pi-hole)
concluídas** — dashboard lista os containers do homelab e mostra o
resumo do Pi-hole (bloqueios, top domínios), com auto-refresh.
Próximo passo: Fase 3 — histórico em SQLite. Ver
[`docs/02-Roadmap.md`](./docs/02-Roadmap.md) e
[`docs/03-Changelog.md`](./docs/03-Changelog.md) para o histórico completo.
