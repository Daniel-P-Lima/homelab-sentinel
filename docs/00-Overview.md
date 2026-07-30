---
tags: [homelab, projeto, rust, vue]
status: em-andamento
criado: 2026-07-30
---

# Homelab Sentinel — Overview

## O que é

Um dashboard unificado para o meu homelab, que agrega e dá inteligência sobre
serviços que já rodam de forma isolada (Docker, Pi-hole, Uptime Kuma, Beszel,
Jellyfin), em vez de eu ter que abrir 5 abas diferentes.

Inspirado no artigo ["The Economic Benefit of Refactoring"](https://martinfowler.com/articles/exploring-gen-ai/refactoring-economic-benefit.html)
(Giles Edwards-Alexander / Thoughtworks) sobre um app pessoal construído
majoritariamente por agentes, com stack Rust + TS. A ideia aqui não é repetir
o domínio dele (gestão pessoal), mas a **forma**: app "sério" com UI reativa,
integrações externas, análise de dados e deploy automatizado — aplicado ao meu
próprio homelab.

## Por que construir isso (e não só usar Grafana/outro dashboard pronto)

- Aprender a stack Rust (axum) + Vue 3 + TS na prática, com um problema real.
- Ter algo feito sob medida pros meus serviços específicos, sem overhead de
  configurar Grafana/Prometheus pra um homelab pequeno.
- Adicionar uma camada de **análise** (não só métricas cruas) — ex: resumo do
  que aconteceu no dia, detecção de padrão estranho em logs.

## Escopo (o que É este projeto)

- Dashboard web único, acessível na rede local, mostrando estado agregado
  dos serviços do homelab.
- Coleta periódica de dados via Docker API, Pi-hole API, etc (background jobs).
- Análise simples de logs/eventos para sinalizar anomalias.
- Notificações externas (Telegram/Discord) quando algo foge do padrão.
- Deploy via Docker Compose, integrado ao stack já existente.

## Fora de escopo (por enquanto)

- Substituir Uptime Kuma ou Beszel — o Sentinel *consome* dados deles ou do
  Docker diretamente, não reimplementa ping/health-check do zero.
- Autenticação multiusuário — é single-user, rede local, atrás do meu próprio
  perímetro.
- Mobile app nativo.

## Ligação com o resto do homelab

Servidor: Ubuntu Server, `192.168.15.32`, usuário `mr_robot`,
diretório `/home/mr_robot/homelab`. Serviços existentes: Pi-hole (8081),
Homer (8082), Uptime Kuma (8083), Firefly III (8084), Beszel (8086),
Jellyfin (8096). Sentinel deve rodar em uma porta livre (sugestão: 8087) e
ser adicionado ao `docker-compose.yml` existente, não substituí-lo.

## Índice de documentos

- [[01-Architecture]] — decisões técnicas e diagrama
- [[02-Roadmap]] — fases e progresso incremental
- [[03-Changelog]] — o que foi feito, sessão a sessão
