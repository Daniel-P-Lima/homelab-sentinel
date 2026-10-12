import { ref, onMounted, onUnmounted, type Ref } from 'vue'

/**
 * Chama `fetcher` ao montar e a cada `intervalMs`, expondo dado/erro/loading.
 *
 * - Um erro passageiro não apaga o último dado bom: `data` só muda quando a
 *   chamada dá certo, e a UI decide se mostra o aviso junto.
 * - Se uma chamada ainda está em voo quando o intervalo dispara, a nova é
 *   pulada — sem isso, um backend lento empilha requests.
 */
export function usePolling<T>(fetcher: () => Promise<T>, intervalMs = 5000) {
  const data = ref<T | null>(null) as Ref<T | null>
  const error = ref<string | null>(null)
  const loading = ref(true)
  let inFlight = false
  let timer: number | undefined

  async function poll() {
    if (inFlight) return
    inFlight = true
    try {
      data.value = await fetcher()
      error.value = null
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'erro desconhecido'
    } finally {
      loading.value = false
      inFlight = false
    }
  }

  onMounted(() => {
    poll()
    timer = window.setInterval(poll, intervalMs)
  })

  onUnmounted(() => {
    if (timer) window.clearInterval(timer)
  })

  return { data, error, loading }
}
