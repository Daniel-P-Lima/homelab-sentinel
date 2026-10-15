/** Um LED do painel do servidor. O `title` explica o estado ao passar o mouse. */
export interface Led {
  label: string
  state: 'on' | 'warn' | 'alert' | 'off'
  title: string
}

/** Classes do `.led` em style.css. Alerta pisca: é o que pede atenção. */
export function ledClass(state: Led['state']): string[] {
  if (state === 'off') return []
  return state === 'alert' ? ['led-alert', 'led-blink'] : [`led-${state}`]
}

/** LED de um container a partir do `state` e do `status` do Docker. */
export function containerLed(state: string, status: string): Led['state'] {
  if (state === 'running') return status.includes('(unhealthy)') ? 'warn' : 'on'
  if (state === 'exited' || state === 'dead') return 'alert'
  return 'warn'
}
