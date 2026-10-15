<script setup lang="ts">
import { computed } from 'vue'
import { usePihole } from '../composables/usePihole'
import { fmtNumber, fmtRelative } from '../format'
import { PIHOLE_WEB_PORT, hostAddress, serviceUrl } from '../links'

const { summary, error, loading } = usePihole()

const adminUrl = serviceUrl(PIHOLE_WEB_PORT, '/admin/')
const adminAddress = hostAddress(PIHOLE_WEB_PORT)

const blockingLabel = computed(() => {
  switch (summary.value?.blocking) {
    case 'enabled':
      return { text: 'Proteção ativa', cls: 'pill-ok' }
    case 'disabled':
      return { text: 'Proteção desligada', cls: 'pill-danger' }
    case 'failed':
      return { text: 'Falha no bloqueio', cls: 'pill-danger' }
    default:
      return { text: 'Estado desconhecido', cls: 'pill-warn' }
  }
})

// a barra de cada domínio é proporcional ao mais bloqueado da lista
const topMax = computed(() => Math.max(1, ...(summary.value?.top_blocked.map((d) => d.count) ?? [])))
</script>

<template>
  <section class="card">
    <header class="card-header">
      <div>
        <h2>🛡️ Pi-hole</h2>
        <a class="service-link" :href="adminUrl" target="_blank" rel="noopener">{{ adminAddress }} ↗</a>
      </div>
      <span v-if="summary" class="pill" :class="blockingLabel.cls">{{ blockingLabel.text }}</span>
    </header>

    <!-- erro aparece junto com o último resumo bom, não no lugar dele -->
    <p v-if="error" class="notice notice-danger">Não foi possível atualizar o Pi-hole: {{ error }}</p>
    <p v-if="loading" class="muted">Carregando Pi-hole...</p>

    <template v-else-if="summary">
      <div class="stats">
        <div class="stat">
          <span class="stat-label">Consultas (24h)</span>
          <span class="stat-value">{{ fmtNumber(summary.queries_total) }}</span>
        </div>
        <div class="stat">
          <span class="stat-label">Bloqueadas</span>
          <span class="stat-value">{{ fmtNumber(summary.queries_blocked) }}</span>
          <div class="bar" :title="`${summary.percent_blocked.toFixed(1)}% bloqueado`">
            <span :style="{ width: `${Math.min(summary.percent_blocked, 100)}%` }" />
          </div>
          <span class="stat-hint">{{ summary.percent_blocked.toFixed(1) }}% do total</span>
        </div>
        <div class="stat">
          <span class="stat-label">Aparelhos ativos</span>
          <span class="stat-value">{{ fmtNumber(summary.clients_active) }}</span>
        </div>
        <div class="stat">
          <span class="stat-label">Domínios na blocklist</span>
          <span class="stat-value">{{ fmtNumber(summary.domains_on_blocklist) }}</span>
          <span v-if="summary.gravity_last_update" class="stat-hint">
            listas atualizadas {{ fmtRelative(summary.gravity_last_update) }}
          </span>
        </div>
      </div>

      <h3>Mais bloqueados</h3>
      <p v-if="summary.queries_total === 0" class="muted empty">
        Nenhuma consulta nas últimas 24h. Os aparelhos da rede já usam o Pi-hole como DNS?
      </p>
      <p v-else-if="summary.top_blocked.length === 0" class="muted empty">
        Nada bloqueado nas últimas 24h.
      </p>
      <ol v-else class="top">
        <li v-for="d in summary.top_blocked" :key="d.domain">
          <div class="top-row">
            <span class="domain" :title="d.domain">{{ d.domain }}</span>
            <span class="muted">{{ fmtNumber(d.count) }}</span>
          </div>
          <div class="bar">
            <span :style="{ width: `${(d.count / topMax) * 100}%`, background: 'var(--danger)' }" />
          </div>
        </li>
      </ol>
    </template>
  </section>
</template>

<style scoped>
.stats {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.75rem;
}
.stat {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  padding: 0.85rem;
  border-radius: 10px;
  background: var(--surface-2);
}
.stat-label {
  font-size: 0.78rem;
  color: var(--text-muted);
}
.stat-value {
  font-size: 1.5rem;
  font-weight: 650;
  font-variant-numeric: tabular-nums;
}
.stat-hint {
  font-size: 0.75rem;
  color: var(--text-muted);
}
h3 {
  margin: 1.25rem 0 0.6rem;
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--text-muted);
}
.empty {
  margin: 0;
  font-size: 0.85rem;
}
.top {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}
.top-row {
  display: flex;
  justify-content: space-between;
  gap: 0.75rem;
  font-size: 0.85rem;
  margin-bottom: 0.25rem;
}
.domain {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: ui-monospace, monospace;
  font-size: 0.8rem;
}
</style>
