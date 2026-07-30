export interface HealthResponse {
  status: string
  uptime_seconds: number
}

export async function fetchHealth(): Promise<HealthResponse> {
  const res = await fetch('/api/health')
  if (!res.ok) {
    throw new Error(`GET /api/health falhou: ${res.status}`)
  }
  return res.json()
}
