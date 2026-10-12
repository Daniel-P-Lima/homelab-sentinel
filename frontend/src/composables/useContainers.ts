import { fetchContainers } from '../api'
import { usePolling } from './usePolling'

export function useContainers() {
  const { data, error, loading } = usePolling(fetchContainers)
  return { containers: data, error, loading }
}
