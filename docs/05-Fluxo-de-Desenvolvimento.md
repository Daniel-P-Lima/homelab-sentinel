---
tags: [homelab, setup, fluxo]
criado: 2026-10-14
atualizado: 2026-10-14
---

# Fluxo de desenvolvimento — Homelab Sentinel

Como o código sai da máquina de casa e chega no servidor, o que cuidar em
cada etapa e o que ainda não está amarrado. Os comandos de setup ficam em
[[04-Setup]]; aqui fica o **porquê** e os pontos de atenção.

## Visão geral

| Onde | Como roda | Para quê |
|------|-----------|----------|
| Mac / Windows (casa) | `cargo run` + `npm run dev` | iterar rápido, hot reload, debug |
| GitHub Actions | `ci.yml` (Rust 1.80, Node 22) | portão obrigatório antes do merge na `main` |
| Servidor `192.168.15.108` | `docker compose up -d --build` | produção: a stack que fica no ar |

```
dev (Mac/Win) ──► checks do CI local ──► push na branch ──► PR ──► CI verde ──► merge na main
                                                                                     │
servidor: git pull ──► docker compose up -d --build ──► conferir logs ◄──────────────┘
```

O fluxo é o padrão para esse tipo de projeto. Em dev, o proxy do Vite
(`/api` → `localhost:8087`, ver `frontend/vite.config.ts`) faz o papel que
o Nginx (`frontend/nginx.conf`) faz em produção. Por isso o frontend usa
caminhos relativos (`/api/...`) e não sabe em qual dos dois ambientes está.

## 1. Desenvolvimento local

### Backend

```bash
cd backend
cargo run      # :8087
```

O `cargo run` **não lê o `.env`**: o projeto não usa `dotenvy`. O `.env` da
raiz é lido só pelo `docker compose`, que o usa para substituir
`${PIHOLE_PASSWORD}` no `docker-compose.yml`. Além disso, o `PIHOLE_URL` nem
está no `.env`: ele fica fixo no compose. Ou seja, em dev as duas variáveis
têm que vir do shell, senão o boot loga:

```
WARN homelab_sentinel::state: PIHOLE_URL/PIHOLE_PASSWORD não configurados. ...
ERROR homelab_sentinel::routes: Pi-hole indisponível para este backend
```

e `/api/pihole/summary` devolve **503**. Isso é o comportamento previsto
para quem roda sem configuração, não um bug (ver [[01-Architecture]]).

```bash
# Mac / Linux (zsh, bash)
export PIHOLE_URL=http://localhost:8081
export PIHOLE_PASSWORD='...'
cargo run
```

```powershell
# Windows (PowerShell): vale só para a sessão atual do terminal
$env:PIHOLE_URL = "http://localhost:8081"
$env:PIHOLE_PASSWORD = "..."
cargo run
```

Para onde apontar o `PIHOLE_URL`:

- **Pi-hole local (`pihole-dev`)**: a interface web está publicada na
  **8081** (não na 8082 do servidor). Fica isolado e pode ser derrubado à
  vontade, o que serve para testar o caminho 502.
- **Pi-hole real (`http://192.168.15.108:8082`)**: tem dados de verdade,
  mas abre uma sessão a mais na API v6, que tem limite de sessões
  simultâneas. Uma sessão de dev não deve ser problema, porque o
  `PiholeClient` reaproveita o `sid`. **Nunca pare o Pi-hole real para
  testar**: ele é o DNS da casa.

O que o boot deve mostrar quando tudo está certo:

```
INFO ... conectado ao Docker daemon
INFO ... conectado ao Pi-hole em http://localhost:8081
```

Se aparecer `Pi-hole em ... não respondeu no boot`, a URL ou a senha está
errada, e o endpoint vai dar **502** (não 503).

**Docker em dev.** Sem `DOCKER_HOST`, o bollard usa o socket local. No Mac
isso exige o Docker Desktop (ou equivalente) rodando; sem ele,
`/api/containers` devolve 503, e isso é esperado. Em dev não existe o
`docker-socket-proxy`, então o backend tem acesso **total** à API do
Docker. Um coletor novo pode funcionar em dev e tomar 403 em produção se
usar um endpoint que o proxy não libera (ver "Armadilhas" no `CLAUDE.md`).

### Frontend

```bash
cd frontend
npm install    # ou npm ci, para instalar exatamente o lock
npm run dev    # :5173, /api proxeado para :8087
```

Se você mexer em dependência, **commite o `package-lock.json` junto**. O CI
e o `Dockerfile` usam `npm ci`, que falha se o lock estiver desatualizado.

## 2. Antes do push: os checks do CI

O CI reprova por qualquer warning (`clippy -D warnings`) e por variável
sobrando no TS (`noUnusedLocals`). Rode antes de subir:

```bash
cd backend  && cargo build && cargo test && cargo clippy --all-targets -- -D warnings
cd frontend && npx vue-tsc -b && npm run build
```

Para ter **certeza** de usar a mesma versão do CI, rode os checks em
containers, com os mesmos comandos do servidor (ver [[04-Setup]]):

```bash
docker run --rm -v "$PWD/backend":/src -w /src rust:1.80-slim \
  sh -c 'rustup component add clippy && cargo test --locked && cargo clippy --locked --all-targets -- -D warnings'
docker run --rm -v "$PWD/frontend":/src:ro node:22-slim \
  sh -c 'cp -r /src /app && cd /app && npm ci && npx vue-tsc -b && npm run build'
```

> [!warning] Versão do toolchain
> Hoje o Rust local é o que estiver instalado (no Mac, em 2026-10-14:
> **1.93.1**), mas o CI e o `Dockerfile` usam **1.80**. Ver a lacuna 1.

## 3. Git, PR e CI

- Uma branch por fase ou assunto, seguindo os prefixos já em uso:
  `feature/fase-N-<tema>`, `fix/<tema>`, `docs/<tema>`.
- PR para a `main`. A branch protection exige `Backend (Rust)` e
  `Frontend (Vue + TS)` verdes.
- Ao fechar uma fase: checkbox em [[02-Roadmap]], entrada em
  [[03-Changelog]] com as **pendências reais**, e [[01-Architecture]] se
  alguma decisão mudou.

## 4. Deploy no servidor

```bash
cd ~/homelab-sentinel
git checkout main && git pull
docker compose up -d --build
docker compose logs -f sentinel-backend   # "conectado ao Docker daemon" e "conectado ao Pi-hole"
```

- **Deploy só da `main`**, depois do merge. Assim o que roda no servidor é
  sempre algo que passou pelo CI.
- **`up -d --build`, não `restart`.** O `restart` não reconstrói a imagem
  nem relê o `.env`: uma senha nova no `.env` só entra quando o container
  é **recriado**, coisa que o `up -d` faz quando detecta a mudança.
- O `.env` do servidor (`chmod 600`) é o único que vale para produção. Ele
  não vai pelo Git; se a senha do Pi-hole mudar, atualize-o e recrie o
  backend.
- O servidor não tem `cargo` nem `npm`. Não edite código nele; se precisar
  validar algo, use os containers da seção 2.

## Boas práticas

- **Paridade de versões**: Rust 1.80 e Node 22 em todos os lugares (dev,
  CI, `Dockerfile`). Mudar de versão é uma decisão única, aplicada nos três
  lugares de uma vez.
- **`--locked` sempre que possível**: o `Cargo.lock` e o
  `package-lock.json` são parte do código. Mudou dependência, commite o
  lock.
- **Teste os caminhos de erro em dev**, não em produção: derrubar o
  `pihole-dev` (502), rodar sem variáveis (503), parar o Docker Desktop
  (503 em `/api/containers`).
- **Segredos só em variável de ambiente ou `.env`**, nunca no código nem
  no histórico do Git. O `.env` está no `.gitignore`.
- **Logs são o primeiro diagnóstico**: o boot diz explicitamente o que
  conectou e o que não conectou.

## Lacunas conhecidas

Coisas que hoje dependem de disciplina, não de configuração. Estão
listadas para decidir uma a uma, sem obrigação de resolver tudo.

1. **Toolchain Rust não está fixado no repo.** Não existe
   `rust-toolchain.toml`, então cada máquina usa o Rust que tiver
   instalado. O risco tem duas formas: uma API estabilizada depois da 1.80
   compila em casa e quebra no CI ou no build da imagem; e o clippy muda a
   cada versão, então com `-D warnings` o que passa em casa pode reprovar
   no CI (e vice-versa). Correção sugerida: `backend/rust-toolchain.toml`
   com `channel = "1.80"` e `components = ["clippy", "rustfmt"]`. O
   `rustup` passa a trocar de versão sozinho ao entrar em `backend/`.
   Antes, confirmar que o código atual passa no clippy da 1.80.
2. **Node local diferente do CI.** O Mac está no Node **25.6.1**; o CI e
   o `Dockerfile` usam o **22**. O risco é menor que no Rust, mas existe
   (comportamento de `npm`, formato do lock). Opções: um `.nvmrc` com `22`
   e/ou `"engines"` no `package.json`.
3. **Variáveis de ambiente em dev são manuais.** É preciso exportar
   `PIHOLE_URL`/`PIHOLE_PASSWORD` a cada terminal novo, com sintaxe
   diferente no Mac e no Windows. Opção: a crate `dotenvy` lendo um
   `backend/.env` de dev (fora do Git). O custo é uma dependência nova
   (`Cargo.lock`) e mudar o fluxo descrito em [[04-Setup]].
4. **O `pihole-dev` não está versionado.** Ele foi criado à mão
   (`pihole/pihole:latest`, portas 8081/5354, sem compose). Em outra
   máquina, como o Windows, não há receita para recriá-lo igual. Opção: um
   `docker-compose.dev.yml` só com o Pi-hole de dev.
5. **Imagens com tag `latest`.** O `tecnativa/docker-socket-proxy:latest`
   no compose (e o `pihole-dev`) podem mudar entre um deploy e outro sem
   que nada no repo mude. Fixar a tag dá um deploy reprodutível.
6. **Fim de linha no Windows.** Não existe `.gitattributes`. Com o
   `core.autocrlf` mal configurado no Windows, arquivos podem ir com CRLF
   e sujar o diff (ou quebrar scripts `sh` dentro de containers). Opção: um
   `.gitattributes` com `* text=auto eol=lf`.
7. **O clippy do CI não usa `--locked`.** `cargo build` e `cargo test`
   usam, `cargo clippy` não. Na prática o lock já foi validado pelos passos
   anteriores, mas é uma inconsistência pequena no `ci.yml`.
8. **Deploy manual.** O servidor só atualiza quando alguém entra por SSH e
   roda `git pull` + `docker compose up`. Para um homelab isso é aceitável;
   automatizar (ex: Watchtower, ou um runner self-hosted) seria uma
   decisão de escopo, a registrar em [[02-Roadmap]] se for o caso.
9. **Sem paridade de rede entre dev e produção.** Em dev não há o
   `docker-socket-proxy` nem a rede `sentinel`. Um coletor que dependa de
   um endpoint novo do Docker só vai mostrar o 403 do proxy no servidor.
   Para validar antes, dá para subir a stack inteira localmente com
   `docker compose up --build`. Só que o `PIHOLE_URL` é fixo no compose
   (`host.docker.internal:8082`), e em casa o Pi-hole de dev está na 8081.
   Então o card do Pi-hole não conecta sem editar o compose ou tornar a URL
   configurável (`${PIHOLE_URL:-...}`).
