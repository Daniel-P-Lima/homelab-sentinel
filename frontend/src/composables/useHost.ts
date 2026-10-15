import { fetchHost } from '../api'
import { usePolling } from './usePolling'

export function useHost() {
  const { data, error, loading } = usePolling(fetchHost)
  return { host: data, error, loading }
}
