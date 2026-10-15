import { computed } from 'vue'
import { fetchContainers, type ContainerInfo } from '../api'
import { SERVICE_CONTAINERS } from '../services'
import { usePolling } from './usePolling'

/**
 * Um poll só, compartilhado pelos cards de serviço e pela tabela: cada
 * `GET /api/containers` custa ~1-2s de stats no daemon, então dois polls
 * dobrariam a carga. Chame uma vez (no App.vue) e repasse por props.
 */
export function useContainers() {
  const { data, error, loading } = usePolling(fetchContainers)

  const byName = computed(() => new Map((data.value ?? []).map((c) => [c.name, c])))

  /** O container de um card: undefined enquanto o primeiro poll não volta. */
  function containerOf(name: string | undefined): ContainerInfo | null | undefined {
    if (!name || data.value === null) return undefined
    return byName.value.get(name) ?? null
  }

  /** Os containers que não têm card próprio. */
  const others = computed(() => data.value?.filter((c) => !SERVICE_CONTAINERS.has(c.name)) ?? null)

  return { containers: data, others, containerOf, error, loading }
}
