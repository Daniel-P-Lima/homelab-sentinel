import { fetchHealth } from '../api'
import { usePolling } from './usePolling'

export function useHealth() {
  const { data, error } = usePolling(fetchHealth)
  return { health: data, error }
}
