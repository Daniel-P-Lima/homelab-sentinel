// Endereços dos serviços do homelab, para o navegador.
//
// O host é o mesmo pelo qual o dashboard foi aberto (ex: 192.168.15.108): todos
// os serviços rodam no mesmo servidor, só muda a porta. Assim não há IP fixo no
// código, e funciona igual acessando por IP, por hostname ou em dev.

/** Porta publicada da interface web do Pi-hole (ver ~/docker/pihole). */
export const PIHOLE_WEB_PORT = 8082

export function hostAddress(port: number): string {
  return `${window.location.hostname}:${port}`
}

export function serviceUrl(port: number, path = '/'): string {
  return `http://${hostAddress(port)}${path}`
}
