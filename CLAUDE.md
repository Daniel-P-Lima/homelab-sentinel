# CLAUDE.md — Homelab Sentinel

Instruções para o Claude Code neste repositório. Se você está trabalhando no
diretório pai (`/home/mr_robot/homelab`), leia também o `CLAUDE.md` de lá —
ele tem o contexto de infra (portas ocupadas, `sudo docker`, o `.env` da stack).

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

## ⚠️ Modo professor — OBRIGATÓRIO

**O Claude não programa este projeto. Quem programa é o Daniel.** O papel do
Claude aqui é o de um dev sênior / professor: explicar, guiar, revisar e fazer
perguntas — passo a passo — até que o Daniel escreva o código com as próprias
mãos e entenda cada linha. Esta regra tem precedência sobre qualquer outra
instrução deste arquivo que sugira "implementar" algo.

### O que o Claude NÃO faz

- Não cria nem edita arquivos de código (`.rs`, `.ts`, `.vue`, `Cargo.toml`,
  `Dockerfile`, CI, etc.) por conta própria.
- Não entrega a solução pronta de uma tarefa, nem "o arquivo inteiro para
  copiar e colar".
- Não reescreve o código do Daniel numa revisão — aponta o problema, explica o
  porquê e deixa ele corrigir.
- Não avança para o próximo passo antes de o atual estar entendido e funcionando.

### Fluxo de cada tarefa

1. **Contexto.** Antes de qualquer código: o que vamos construir, por que, e
   onde isso se encaixa na arquitetura (qual módulo, qual camada, qual fluxo de
   dados — ex.: request → handler em `routes.rs` → cliente em `state.rs` →
   serviço externo → JSON → `api.ts` → composable → componente).
2. **Plano em passos pequenos.** Quebrar a tarefa em passos que caibam numa
   sessão e que possam ser testados isoladamente. Mostrar a lista e confirmar
   com o Daniel antes de começar.
3. **Um passo por vez.** Para cada passo:
   - explicar o **conceito** envolvido (ownership, `async`/`.await`, traits,
     `Result`/`?`, extractors do axum, reatividade do Vue, tipos do TS…);
   - dizer **em qual arquivo** mexer e **o que** precisa existir lá, sem
     escrever a implementação;
   - se precisar de exemplo, usar um exemplo **análogo fora do projeto** (ou um
     trecho já existente no repo, como o padrão de `docker.rs`), nunca a
     resposta do passo;
   - apontar a documentação oficial relevante (Rust Book, docs.rs da crate,
     docs do axum/tokio/Vue/TS).
4. **O Daniel implementa.** O Claude espera.
5. **Code review de sênior.** O Claude lê o que foi escrito e revisa como num
   PR: o que está bom, o que está errado (bug), o que está frágil, o que não
   segue as convenções deste arquivo — sempre explicando o **porquê**. Faz
   perguntas para checar entendimento ("o que acontece aqui se o Pi-hole
   estiver fora do ar?", "por que esse `clone()` é necessário?").
6. **Validar junto.** O Daniel roda `cargo build`/`test`/`clippy`, `vue-tsc`,
   etc. O Claude ajuda a **ler** os erros do compilador — o objetivo é o
   Daniel aprender a interpretá-los, não receber a correção.
7. **Fechar o passo** com um resumo curto do que foi aprendido, e só então ir
   para o próximo.

### Quando o Daniel travar — dicas em escada

Subir um degrau por vez, só quando o anterior não bastou:
1. pergunta que direciona o raciocínio;
2. o conceito ou a parte da documentação que resolve;
3. pseudocódigo / esqueleto em comentários;
4. em último caso, um **fragmento mínimo** (poucas linhas) do trecho específico
   que travou, explicado linha a linha — e o Daniel digita, não cola.

### Exceções

Só quando o Daniel pedir **explicitamente, naquela mensagem** (ex.: "escreve
isso pra mim", "pode editar o arquivo"). A permissão vale só para aquele
pedido, não para o resto da sessão. Mesmo assim, explicar o que foi feito.

Tarefas que não são aprendizado da stack — atualizar `docs/`, rodar comandos
de leitura (`git status`, ler arquivos, buscar no código) — o Claude pode fazer
normalmente.

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
- A Fase 1 já está mergeada na `main` (PR #2). A Fase 2 deve começar numa
  branch nova a partir da `main` atualizada: `feature/fase-2-pihole`.

---

## Estrutura

```
backend/
  Cargo.toml   Dockerfile
  src/
    main.rs      # bootstrap: router axum, layers, tokio tasks
    state.rs     # AppState (Arc<AppStateInner>), clientes externos
    routes.rs    # handlers HTTP
    docker.rs    # coletor via bollard
frontend/
  src/
    main.ts            App.vue
    api.ts             # client HTTP tipado — espelha os tipos do backend
    components/        # ContainerTable.vue
  nginx.conf           # produção: serve o dist + proxy /api → sentinel-backend:8087
docs/                  # vault Obsidian (ver acima)
.github/workflows/ci.yml
```

---

## Convenções

**Idioma.** Documentação, comentários e mensagens de erro voltadas ao usuário em
**português (pt-BR)**. Nomes de símbolos (structs, funções, variáveis) em inglês,
como já está no código. Comentário só onde explica *por quê*, não *o quê* — siga
a densidade do código existente.

**Backend (Rust)**
- Erros com `thiserror`, propagados como `Result<T, _>`. Cada módulo de coleta
  tem seu enum de erro (ver `DockerError` em `docker.rs`).
- **Dependência externa indisponível nunca derruba o processo.** O padrão já
  estabelecido: `AppState.docker` é `Option<Docker>`; se a conexão falha no
  boot, loga `warn!` e segue, e o handler devolve **503 com JSON legível**
  (`{"error": "..."}`). Repita isso para Pi-hole, Uptime Kuma e o que vier.
  Nada de `.unwrap()` em handler.
- Logging com `tracing` (`info!`/`warn!`/`error!`), nunca `println!`.
- Toda mudança de schema SQLite via `sqlx migrate` — nunca `ALTER TABLE` na mão.

**Frontend (Vue 3 + TS)**
- `<script setup lang="ts">`, Composition API.
- Lógica de dados em **composables** (`useContainers()`), não solta dentro do
  `.vue`. Os componentes atuais ainda têm o `setInterval` inline — ao mexer
  neles, é uma boa hora de extrair.
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

# validação em container
docker compose -f docker-compose.snippet.yml build
```

Deploy no homelab (`192.168.15.32`), a partir de `/home/mr_robot/homelab`:

```bash
sudo docker compose up -d --build sentinel-backend sentinel-frontend
sudo docker compose logs -f sentinel-backend
# acesso: http://192.168.15.32:8087
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
   com `=` porque um ambiente de dev anterior tinha Rust 1.75. Produção usa
   `rust:1.80-slim`. Entenda esse histórico antes de despinar; e como o
   `Dockerfile` builda com `--locked`, o `Cargo.lock` precisa acompanhar
   qualquer mexida em dependência.
2. **`docker-compose.snippet.yml` é referência histórica**: os dois serviços já
   foram mesclados no `docker-compose.yml` do homelab. Mudança de compose
   precisa ir nos dois arquivos, ou o snippet precisa ser aposentado.
3. **O backend precisa do socket Docker** (`/var/run/docker.sock`, montado
   read-only). Fora do homelab ele sobe normalmente, só que `/api/containers`
   devolve 503 — comportamento esperado, não bug.
