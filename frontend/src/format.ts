const numberFmt = new Intl.NumberFormat('pt-BR')

export function fmtNumber(v: number): string {
  return numberFmt.format(v)
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
  return mb >= 1024 ? `${(mb / 1024).toFixed(1)} GB` : `${mb.toFixed(0)} MB`
}
