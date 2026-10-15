// Serviços fixos do homelab: cada um ganha um card no topo do dashboard.
// Os containers que não estão aqui vão para a tabela "Outros containers".
//
// O endereço principal é o HTTPS servido pelo Caddy (~/docker/caddy), que só
// resolve em aparelhos com o Tailscale ligado. O `ip:porta` fica como acesso
// direto, para quando o Caddy cair.

import { hostAddress, serviceUrl } from './links'

/** Domínio do coringa `*.fsocietyserver.duckdns.org` no Caddyfile. */
export const SERVICES_DOMAIN = 'fsocietyserver.duckdns.org'

export interface Service {
  /** Nome do container no Docker. Sem ele, o serviço roda fora do Docker e o card só tem os links. */
  container?: string
  name: string
  icon: string
  description: string
  /** Subdomínio no Caddy: `jellyfin` → https://jellyfin.fsocietyserver.duckdns.org */
  subdomain: string
  path?: string
  /** Porta publicada no host, para o acesso direto. */
  port: number
  /** O Cockpit serve HTTPS próprio na porta direta; os outros, HTTP. */
  directScheme?: 'http' | 'https'
}

/** Fora de `SERVICES`: o Pi-hole já tem o card grande de estatísticas, que usa estes endereços. */
export const PIHOLE_SERVICE: Service = {
  container: 'pihole',
  name: 'Pi-hole',
  icon: '🛡️',
  description: 'DNS e bloqueio de anúncios',
  subdomain: 'pihole',
  path: '/admin/',
  port: 8082,
}

export const SERVICES: Service[] = [
  {
    container: 'jellyfin',
    name: 'Jellyfin',
    icon: '🎬',
    description: 'Filmes e séries',
    subdomain: 'jellyfin',
    port: 8096,
  },
  {
    container: 'qbittorrent',
    name: 'qBittorrent',
    icon: '⬇️',
    description: 'Downloads',
    subdomain: 'qbit',
    port: 8081,
  },
  {
    container: 'filebrowser',
    name: 'File Browser',
    icon: '📁',
    description: 'Arquivos do servidor',
    subdomain: 'files',
    port: 8080,
  },
  {
    name: 'Cockpit',
    icon: '🖥️',
    description: 'Administração do servidor',
    subdomain: 'cockpit',
    port: 9090,
    directScheme: 'https',
  },
]

/** Containers que já têm card (incluindo o do Pi-hole) — a tabela mostra só o resto. */
export const SERVICE_CONTAINERS = new Set(
  [...SERVICES, PIHOLE_SERVICE].flatMap((s) => (s.container ? [s.container] : [])),
)

export function httpsHost(s: Service): string {
  return `${s.subdomain}.${SERVICES_DOMAIN}`
}

export function httpsUrl(s: Service): string {
  return `https://${httpsHost(s)}${s.path ?? '/'}`
}

export function directAddress(s: Service): string {
  return hostAddress(s.port)
}

export function directUrl(s: Service): string {
  return serviceUrl(s.port, s.path, s.directScheme)
}
