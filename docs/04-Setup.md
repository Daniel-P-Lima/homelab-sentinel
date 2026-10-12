---
tags: [homelab, setup]
atualizado: 2026-10-07
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

## Deploy no homelab (`192.168.15.108`)

O repo fica clonado em `/home/mr_robot/homelab-sentinel` e o
`docker-compose.yml` da raiz sobe a stack completa (docker-proxy +
backend + frontend) numa rede própria. O usuário `mr_robot` está no
grupo `docker`, então não precisa de `sudo`.

```bash
cd ~/homelab-sentinel
git pull
docker compose up -d --build
docker compose logs -f sentinel-backend   # deve aparecer "conectado ao Docker daemon"
```


Acesso de qualquer máquina da rede: `http://192.168.15.108:8087`.

O servidor não tem `cargo`/`npm`; para rodar o que o CI cobra antes de
subir, usar containers (mesmas versões do CI/Dockerfile):

```bash
docker run --rm -v "$PWD/backend":/src -w /src rust:1.80-slim \
  sh -c 'rustup component add clippy && cargo test --locked && cargo clippy --locked --all-targets -- -D warnings'
docker run --rm -v "$PWD/frontend":/src:ro node:22-slim \
  sh -c 'cp -r /src /app && cd /app && npm ci && npx vue-tsc -b && npm run build'
```

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
- O backend não monta o socket Docker: quem monta é o
  `docker-proxy` (`tecnativa/docker-socket-proxy`), que só libera `GET` em
  `/containers` e `/_ping`. Ver [[01-Architecture]].
- Nenhum dado sensível deve ir para o SQLite sem criptografia se, no
  futuro, o Sentinel passar a lidar com credenciais (ex: tokens de API).
