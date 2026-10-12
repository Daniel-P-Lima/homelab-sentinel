import { fetchPiholeSummary } from '../api'
import { usePolling } from './usePolling'

export function usePihole() {
  const { data, error, loading } = usePolling(fetchPiholeSummary)
  return { summary: data, error, loading }
}
