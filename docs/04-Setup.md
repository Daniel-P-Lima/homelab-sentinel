---
tags: [homelab, setup]
atualizado: 2026-07-30
---

# Setup — Homelab Sentinel

## Rodando em desenvolvimento (na sua máquina, não no server)

**Backend:**
```bash
cd backend
cargo run
# escutando em http://localhost:8087
```

**Frontend:**
```bash
cd frontend
npm install
npm run dev
# escutando em http://localhost:5173, com /api proxeado pro backend
```

## Rodando via Docker (validação local antes de ir pro homelab)

```bash
docker compose -f docker-compose.snippet.yml build
docker compose -f docker-compose.snippet.yml up
```

## Deploy no homelab (`192.168.15.32`)

1. Copiar a pasta `homelab-sentinel/` para `/home/mr_robot/homelab/`
   (via `scp -r homelab-sentinel mr_robot@192.168.15.32:~/homelab/`).
2. Mesclar o conteúdo de `docker-compose.snippet.yml` no
   `docker-compose.yml` já existente (ajustar o nome da `network`).
3. `docker compose up -d --build sentinel-backend sentinel-frontend`
4. Acessar `http://192.168.15.32:8087`.
5. (Opcional) Adicionar o link no Homer.

## Git / GitHub

O projeto já vem com `git init` feito e o commit inicial (Fase 0 + 0.5).
Pra conectar ao GitHub:

```bash
cd homelab-sentinel
git remote add origin git@github.com:<seu-usuario>/homelab-sentinel.git
git branch -M main
git push -u origin main
```

Depois disso:
1. Ativar branch protection na `main` (Settings → Branches → require
   status checks: `Backend (Rust)` e `Frontend (Vue + TS)`).
2. O workflow `.github/workflows/ci.yml` já roda sozinho em todo push/PR.
3. Trabalhar em branches por feature (`feature/coletor-docker`) e abrir
   PR pra `main` — o CI vai validar antes do merge.

## Notas
- O backend precisa de acesso ao socket Docker (`/var/run/docker.sock`)
  para o coletor da Fase 1 — já mapeado no snippet, montado **read-only**.
- Nenhum dado sensível deve ir para o SQLite sem criptografia se, no
  futuro, o Sentinel passar a lidar com credenciais (ex: tokens de API).
