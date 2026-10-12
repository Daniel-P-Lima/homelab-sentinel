<script setup lang="ts">
import { computed } from 'vue'
import { useContainers } from '../composables/useContainers'
import { fmtMb } from '../format'

const { containers, error, loading } = useContainers()

const runningCount = computed(() => containers.value?.filter((c) => c.state === 'running').length ?? 0)

function stateClass(state: string): string {
  if (state === 'running') return 'pill-ok'
  if (state === 'exited' || state === 'dead') return 'pill-danger'
  return 'pill-warn'
}

function fmtPercent(v: number | null): string {
  return v === null ? '—' : `${v.toFixed(1)}%`
}

function fmtMem(usage: number | null, limit: number | null): string {
  if (usage === null) return '—'
  return limit === null ? fmtMb(usage) : `${fmtMb(usage)} / ${fmtMb(limit)}`
}

function memPercent(usage: number | null, limit: number | null): number | null {
  return usage === null || !limit ? null : Math.min((usage / limit) * 100, 100)
}
</script>

<template>
  <section class="card">
    <header class="card-header">
      <h2>🐳 Containers</h2>
      <span v-if="containers" class="pill" :class="runningCount === containers.length ? 'pill-ok' : 'pill-warn'">
        {{ runningCount }} de {{ containers.length }} rodando
      </span>
    </header>

    <!-- erro aparece junto com a última tabela boa, não no lugar dela -->
    <p v-if="error" class="notice notice-danger">Não foi possível atualizar os containers: {{ error }}</p>
    <p v-if="loading" class="muted">Carregando containers... (a primeira leitura de CPU leva ~2s)</p>
    <p v-else-if="containers && containers.length === 0" class="muted">Nenhum container encontrado.</p>

    <div v-else-if="containers" class="table-wrap">
      <table>
        <thead>
          <tr>
            <th>Container</th>
            <th>Estado</th>
            <th class="num">CPU</th>
            <th>Memória</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="c in containers" :key="c.id">
            <td>
              <div class="name">{{ c.name }}</div>
              <div class="image" :title="c.image">{{ c.image }}</div>
            </td>
            <td>
              <span class="pill" :class="stateClass(c.state)">{{ c.status || c.state }}</span>
            </td>
            <td class="num">{{ fmtPercent(c.cpu_percent) }}</td>
            <td class="mem">
              <span>{{ fmtMem(c.mem_usage_mb, c.mem_limit_mb) }}</span>
              <div v-if="memPercent(c.mem_usage_mb, c.mem_limit_mb) !== null" class="bar">
                <span :style="{ width: `${memPercent(c.mem_usage_mb, c.mem_limit_mb)}%` }" />
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
</template>

<style scoped>
/* em tela estreita a tabela rola sozinha em vez de estourar a página */
.table-wrap {
  overflow-x: auto;
  margin: 0 -0.25rem;
}
table {
  width: 100%;
  border-collapse: collapse;
}
th,
td {
  text-align: left;
  padding: 0.65rem 0.5rem;
  border-bottom: 1px solid var(--border);
  font-size: 0.88rem;
  vertical-align: middle;
}
tbody tr:last-child td {
  border-bottom: none;
}
tbody tr:hover {
  background: var(--surface-2);
}
th {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.03em;
}
.num {
  text-align: right;
  font-variant-numeric: tabular-nums;
}
.name {
  font-weight: 600;
}
.image {
  max-width: 16rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-muted);
  font-family: ui-monospace, monospace;
  font-size: 0.75rem;
}
.mem {
  min-width: 9rem;
  font-variant-numeric: tabular-nums;
}
.mem .bar {
  margin-top: 0.3rem;
}
</style>
