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
