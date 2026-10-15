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

## 2026-10-12 — Fase 2: integração Pi-hole + interface nova
- `backend/src/pihole.rs`: cliente da API v6 (`reqwest`). Login com
  `POST /api/auth`, `sid` guardado e reaproveitado, relogin automático em
  401. O resumo junta `/api/stats/summary`, `/api/stats/top_domains?blocked=true`
  e `/api/dns/blocking`, buscados em paralelo (`tokio::try_join!`).
- Endpoint `GET /api/pihole/summary`. Sem config → 503; erro falando com
  o Pi-hole → 502 (`AppError::Pihole`). Ver [[01-Architecture]].
- Config por `PIHOLE_URL` / `PIHOLE_PASSWORD`; a senha vem de um `.env`
  fora do Git (`.env.example` versionado). Backend alcança o Pi-hole por
  `host.docker.internal:8082`.
- 4 testes novos com respostas reais da API (formato do resumo, gravity
  nunca rodado, login válido e senha errada).
- Frontend: `PiholeCard.vue` (estado da proteção, consultas/bloqueadas
  nas últimas 24h, aparelhos ativos, domínios na blocklist, top 5
  bloqueados) com `usePihole()` em cima do `usePolling`.
- Interface redesenhada: layout em cards (Pi-hole + containers lado a
  lado, empilhando no celular), tema claro/escuro seguindo o sistema,
  uptime legível ("2h 15m"), números em pt-BR, contador "X de Y rodando",
  barra de uso de memória, tabela com rolagem horizontal em tela estreita.
- **Validado no homelab:** `docker compose up -d --build`; log mostra
  "conectado ao Pi-hole"; `/api/pihole/summary` devolve os dados reais;
  três polls seguidos abrem **uma** sessão só; derrubando a sessão pelo
  lado do Pi-hole, o próximo poll reloga e responde 200. `cargo test`
  (10 testes), `clippy -D warnings`, `vue-tsc` e `vite build` rodados em
  containers com as versões do CI.
- **Pendências:**
  - A interface nova só foi validada por build e pelo HTML servido —
    falta abrir num navegador (desktop e celular, claro e escuro).
  - O caminho 502 (Pi-hole fora do ar) não foi testado ao vivo: parar o
    Pi-hole derruba o DNS da casa.
  - O Pi-hole ainda está com a senha placeholder `TROQUE_ESTA_SENHA`.
    Ao trocar, atualizar também o `.env` do Sentinel e recriar o backend.

## 2026-10-12 — Endereços de acesso no dashboard
- Backend: `ContainerInfo` ganhou `ports` (portas publicadas no host, vindas
  do próprio `list_containers` — nenhuma chamada nova ao Docker, então o
  docker-proxy não precisa de permissão extra). `published_ports()` tira a
  duplicata IPv4/IPv6, as portas presas ao loopback e as só expostas
  internamente; 1 teste novo.
- Frontend: coluna "Acesso" na tabela de containers (porta TCP vira link
  `ip:porta`; UDP aparece como texto, ex: `53/udp`) e link para
  `/admin/` no card do Pi-hole. O IP é o `window.location.hostname` — o
  mesmo pelo qual o dashboard foi aberto —, sem IP fixo no código
  (`src/links.ts`). A porta do Pi-hole (8082) fica fixa lá. 


## 2026-10-14 — Docs: fluxo de desenvolvimento
- Novo [[05-Fluxo-de-Desenvolvimento]]: dev local (`cargo run` +
  `npm run dev` no Mac/Windows) → checks do CI → PR → deploy com
  `docker compose` no servidor, com boas práticas e lacunas conhecidas.
- Origem: um `cargo run` no Mac logou `PIHOLE_URL/PIHOLE_PASSWORD não
  configurados` e o card deu 503. Causa: o backend não lê o `.env`
  (quem lê é só o compose) e o `PIHOLE_URL` existe apenas no
  `docker-compose.yml`. [[04-Setup]] agora diz isso explicitamente.
- Só documentação: nenhuma das lacunas listadas foi corrigida.
- **Pendências:**
  - Decidir as lacunas uma a uma (a mais relevante é o toolchain Rust sem
    pin: 1.93.1 no Mac contra 1.80 no CI/Dockerfile).
  - O fluxo do Windows (sintaxe PowerShell, fim de linha) foi escrito sem
    testar numa máquina Windows.

## 2026-10-15 — Cards dos serviços e endereços HTTPS
- Os serviços fixos do homelab (Jellyfin, qBittorrent, File Browser e
  Cockpit) ganharam um card cada no topo: estado, CPU, memória, link
  HTTPS pelo Caddy (`https://<serviço>.fsocietyserver.duckdns.org`) e o
  `ip:porta` direto como reserva para quando o Caddy cair. O Cockpit não é
  container, então o card dele só tem os links.
- A lista fica em `frontend/src/services.ts` (container ↔ subdomínio ↔
  porta). Container fora dela vai para a tabela, agora "Outros containers".
- `useContainers()` passou a ser chamado uma vez só, no `App.vue`, que
  repassa os dados por props aos cards e à tabela: dois polls dobrariam as
  chamadas de stats, que custam ~1-2s cada no daemon.
- O Pi-hole não ganhou card pequeno (repetiria o card de estatísticas): o
  card grande passou a ter o link HTTPS, o acesso direto e uma linha
  pequena com estado, CPU e memória do container (que sai da tabela).
- Links para os serviços com `rel="noopener noreferrer"`: o qBittorrent
  respondia 401 "Unauthorized" ao clique, porque a proteção CSRF dele recusa
  `Referer` de outra origem (reproduzido com `curl -H Referer`).
- Favicon (`frontend/public/favicon.svg`): escudo com linha de pulso.
- Sem mudança no backend. `vue-tsc -b` e `npm run build` rodados num
  container `node:22-slim`; frontend redeployado no homelab e conferido pelo
  bundle servido.
- **Pendências:**
  - Não foi aberto num navegador: falta ver o layout no desktop e no
    celular, claro e escuro.
  - Os links HTTPS só abrem com o Tailscale ligado (o DuckDNS aponta pro IP
    100.x). Na LAN sem Tailscale, só o acesso direto funciona.
  - O domínio está fixo em `services.ts`; trocar de domínio exige rebuild.

## 2026-10-15 — Card do servidor
- Backend: `host.rs` + `GET /api/host` com CPU% (delta de duas leituras do
  `/proc/stat`, 500ms), carga, memória e swap (`/proc/meminfo`), uptime,
  temperatura do processador (zona `x86_pkg_temp`), bateria (o servidor é
  um notebook: capacidade, estado e saúde) e disco via `statvfs`. Sem
  `/proc` (Mac) o endpoint dá 503, no mesmo padrão do Docker. 8 testes novos
  com recortes reais do servidor; dependência nova: `libc` (já estava no
  lock como transitiva).
- Compose: `~/media` montado só-leitura em `/host/disk` (`HOST_DISK_PATH`).
- Frontend: `HostCard.vue` no topo, com barras que ficam amarelas/vermelhas
  por nível de uso e um aviso quando a bateria está descarregando (= faltou
  energia em casa).
- Validado: `cargo build`/`test`/`clippy -D warnings` no `rust:1.80-slim`,
  `vue-tsc` + `vite build` no `node:22-slim`, stack redeployada e o JSON do
  `/api/host` conferido com `df`, `/proc/uptime` e `/sys` no servidor.
- **Pendências:**
  - Não aberto num navegador (layout desktop/celular, claro/escuro).
  - O aviso de "na bateria" não foi testado ao vivo (exige tirar o
    notebook da tomada).
  - O disco mostra ~98 GB porque o volume LVM do Ubuntu foi criado com
    100 GB de um SSD de 447 GB; ao estender (`lvextend -r`), o card acompanha
    sem mudança de código.

## 2026-10-15 — Visual "painel de aparelho"
- Frontend redesenhado com o plugin `frontend-design`: o servidor vira um
  equipamento de alumínio escovado (grafite anodizado no tema escuro), com
  LEDs de status reais (Energia, Sentinel, Docker, DNS), indicador de
  bateria em células e quatro medidores de ponteiro (`Meter.vue`, SVG) para
  CPU, memória, disco e temperatura.
- Serviços num rack único com divisórias (canais), em vez de cards soltos;
  botão "Abrir <serviço>" e acesso direto embaixo. Pi-hole e tabela viram
  visores rebaixados. Emojis, pills e fonte monoespaçada saíram.
- Tipografia: IBM Plex Sans Condensed, servida pelo próprio dashboard
  (`@fontsource`, sem Google Fonts). Números com vírgula decimal em toda a
  página (`fmtDecimal`); estado do Docker traduzido ("No ar há 2 horas").
- `usePihole()` subiu para o `App.vue` como os containers: o LED de DNS e o
  card do Pi-hole usam o mesmo poll.
- Movimento: só o ponteiro (sai do zero na primeira leitura e acompanha os
  dados) e o LED de alerta piscando; `prefers-reduced-motion` desliga ambos.
- Validado: `vue-tsc` + `vite build` no `node:22-slim`; redeploy e
  screenshots com Chrome headless em desktop claro, desktop escuro e
  celular (390px).
- **Pendências:**
  - Não testado num navegador real com interação (foco por teclado,
    hover); só screenshots.
  - O LED de alerta piscando e o aviso de bateria não foram vistos ao vivo.

<!-- Próxima entrada: Fase 3 — background jobs + persistência histórica -->
