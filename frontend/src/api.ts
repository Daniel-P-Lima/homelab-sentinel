export interface HealthResponse {
  status: string
  uptime_seconds: number
}

export interface ContainerInfo {
  id: string
  name: string
  image: string
  state: string
  status: string
  cpu_percent: number | null
  mem_usage_mb: number | null
  mem_limit_mb: number | null
  ports: PortMapping[]
}

/** Espelha `PortMapping` em backend/src/docker.rs: só portas publicadas no host. */
export interface PortMapping {
  public_port: number
  private_port: number
  protocol: string
}

export async function fetchHealth(): Promise<HealthResponse> {
  const res = await fetch('/api/health')
  if (!res.ok) {
    throw new Error(`GET /api/health falhou: ${res.status}`)
  }
  return res.json()
}

export async function fetchContainers(): Promise<ContainerInfo[]> {
  const res = await fetch('/api/containers')
  if (!res.ok) {
    const body = await res.json().catch(() => null)
    throw new Error(body?.error ?? `GET /api/containers falhou: ${res.status}`)
  }
  return res.json()
}

export interface TopDomain {
  domain: string
  count: number
}

/** Espelha `PiholeSummary` em backend/src/pihole.rs. Queries = últimas 24h. */
export interface PiholeSummary {
  blocking: 'enabled' | 'disabled' | 'failed' | 'unknown'
  queries_total: number
  queries_blocked: number
  percent_blocked: number
  unique_domains: number
  clients_active: number
  domains_on_blocklist: number
  gravity_last_update: number | null
  top_blocked: TopDomain[]
}

/** Espelha `HostSummary` em backend/src/host.rs. */
export interface HostSummary {
  uptime_seconds: number
  cpu_count: number
  cpu_percent: number | null
  load_avg: [number, number, number]
  mem_total_mb: number
  mem_used_mb: number
  swap_total_mb: number
  swap_used_mb: number
  disk: DiskUsage | null
  temperature_c: number | null
  battery: Battery | null
}

export interface DiskUsage {
  total_gb: number
  used_gb: number
  available_gb: number
  used_percent: number
}

export interface Battery {
  capacity_percent: number
  status: string
  health_percent: number | null
}

export async function fetchHost(): Promise<HostSummary> {
  const res = await fetch('/api/host')
  if (!res.ok) {
    const body = await res.json().catch(() => null)
    throw new Error(body?.error ?? `GET /api/host falhou: ${res.status}`)
  }
  return res.json()
}

export async function fetchPiholeSummary(): Promise<PiholeSummary> {
  const res = await fetch('/api/pihole/summary')
  if (!res.ok) {
    const body = await res.json().catch(() => null)
    throw new Error(body?.error ?? `GET /api/pihole/summary falhou: ${res.status}`)
  }
  return res.json()
}
