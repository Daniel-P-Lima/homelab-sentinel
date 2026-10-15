const numberFmt = new Intl.NumberFormat('pt-BR')

export function fmtNumber(v: number): string {
  return numberFmt.format(v)
}

/** Número com casas decimais fixas e vírgula: 1.5 → "1,5". */
export function fmtDecimal(v: number, digits: number): string {
  return v.toLocaleString('pt-BR', { minimumFractionDigits: digits, maximumFractionDigits: digits })
}

/** 93784 → "1d 2h 3m". Mostra no máximo as duas maiores unidades. */
export function fmtDuration(totalSeconds: number): string {
  const d = Math.floor(totalSeconds / 86400)
  const h = Math.floor((totalSeconds % 86400) / 3600)
  const m = Math.floor((totalSeconds % 3600) / 60)
  const units: [number, string][] = [
    [d, 'd'],
    [h, 'h'],
    [m, 'm'],
  ]
  const parts = units.filter(([n]) => n > 0)
  if (parts.length === 0) return `${Math.floor(totalSeconds)}s`
  return parts
    .slice(0, 2)
    .map(([n, u]) => `${n}${u}`)
    .join(' ')
}

/** Unix timestamp → "há 2 dias", relativo a agora. */
export function fmtRelative(unixSeconds: number): string {
  const rtf = new Intl.RelativeTimeFormat('pt-BR', { numeric: 'auto' })
  const diff = unixSeconds - Date.now() / 1000
  const units: [Intl.RelativeTimeFormatUnit, number][] = [
    ['day', 86400],
    ['hour', 3600],
    ['minute', 60],
  ]
  for (const [unit, secs] of units) {
    if (Math.abs(diff) >= secs) return rtf.format(Math.round(diff / secs), unit)
  }
  return 'agora mesmo'
}

export function fmtMb(mb: number): string {
  return mb >= 1024 ? `${fmtDecimal(mb / 1024, 1)} GB` : `${fmtDecimal(mb, 0)} MB`
}

export function fmtPercent(v: number | null): string {
  return v === null ? '—' : `${fmtDecimal(v, 1)}%`
}

export function fmtMem(usage: number | null, limit: number | null): string {
  if (usage === null) return '—'
  return limit === null ? fmtMb(usage) : `${fmtMb(usage)} / ${fmtMb(limit)}`
}

export function memPercent(usage: number | null, limit: number | null): number | null {
  return usage === null || !limit ? null : Math.min((usage / limit) * 100, 100)
}

const DOCKER_UNITS: Record<string, [string, string]> = {
  second: ['segundo', 'segundos'],
  minute: ['minuto', 'minutos'],
  hour: ['hora', 'horas'],
  day: ['dia', 'dias'],
  week: ['semana', 'semanas'],
  month: ['mês', 'meses'],
  year: ['ano', 'anos'],
}

/** "2 hours", "About an hour", "Less than a second" (formato do Docker) → pt-BR. */
function translateDockerDuration(d: string): string {
  if (/^less than a second$/i.test(d)) return 'menos de um segundo'
  const about = d.match(/^about an? (\w+)$/i)
  if (about) {
    const unit = DOCKER_UNITS[about[1].toLowerCase()]
    return unit ? `cerca de ${unit[0] === 'hora' || unit[0] === 'semana' ? 'uma' : 'um'} ${unit[0]}` : d
  }
  const n = d.match(/^(\d+) (\w+?)s?$/)
  if (n) {
    const unit = DOCKER_UNITS[n[2].toLowerCase()]
    if (unit) return `${n[1]} ${n[1] === '1' ? unit[0] : unit[1]}`
  }
  return d
}

/** Texto de estado de um container em pt-BR, a partir do `state` e do `status` do Docker. */
export function fmtContainerStatus(state: string, status: string): string {
  if (state === 'running') {
    const up = status.match(/^Up (.+?)(?: \((?:healthy|unhealthy|health: starting)\))?$/)
    const base = up ? `No ar há ${translateDockerDuration(up[1])}` : 'No ar'
    return status.includes('(unhealthy)') ? `${base}, health-check falhando` : base
  }
  const states: Record<string, string> = {
    exited: 'Parado',
    dead: 'Parado com erro',
    paused: 'Pausado',
    restarting: 'Reiniciando',
    created: 'Criado, não iniciado',
  }
  return states[state] ?? state
}
