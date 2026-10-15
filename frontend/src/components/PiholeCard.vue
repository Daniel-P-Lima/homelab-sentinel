<script setup lang="ts">
import { computed } from 'vue'
import type { ContainerInfo, PiholeSummary } from '../api'
import { fmtContainerStatus, fmtDecimal, fmtMb, fmtNumber, fmtPercent, fmtRelative } from '../format'
import { ledClass, type Led } from '../leds'
import { PIHOLE_SERVICE, directAddress, directUrl, httpsUrl } from '../services'

// Dados vêm do App.vue: o LED de DNS no painel do servidor usa o mesmo poll.
const props = defineProps<{
  summary: PiholeSummary | null
  error: string | null
  loading: boolean
  /** undefined = ainda carregando; null = o container não existe. */
  container: ContainerInfo | null | undefined
}>()

const blocking = computed<{ text: string; led: Led['state'] }>(() => {
  switch (props.summary?.blocking) {
    case 'enabled':
      return { text: 'Bloqueando anúncios', led: 'on' }
    case 'disabled':
      return { text: 'Bloqueio desligado', led: 'warn' }
    case 'failed':
      return { text: 'Falha no bloqueio', led: 'alert' }
    default:
      return { text: 'Estado do bloqueio desconhecido', led: 'off' }
  }
})

// a barra de cada domínio é proporcional ao mais bloqueado da lista
const topMax = computed(() => Math.max(1, ...(props.summary?.top_blocked.map((d) => d.count) ?? [])))
</script>

<template>
  <section class="display pihole" aria-labelledby="pihole-title">
    <header class="section-title">
      <h2 id="pihole-title">Pi-hole</h2>
      <span v-if="summary" class="state">
        <span class="led" :class="ledClass(blocking.led)" />
        {{ blocking.text }}
      </span>
    </header>

    <p v-if="error" class="notice notice-danger">Sem resposta do Pi-hole: {{ error }}</p>
    <p v-if="loading" class="muted">Lendo o Pi-hole...</p>

    <template v-else-if="summary">
      <dl class="figures">
        <div>
          <dt>Consultas nas últimas 24h</dt>
          <dd>{{ fmtNumber(summary.queries_total) }}</dd>
        </div>
        <div>
          <dt>Bloqueadas</dt>
          <dd>{{ fmtNumber(summary.queries_blocked) }}</dd>
          <span class="hint">{{ fmtDecimal(summary.percent_blocked, 1) }}% do total</span>
        </div>
        <div>
          <dt>Aparelhos ativos</dt>
          <dd>{{ fmtNumber(summary.clients_active) }}</dd>
        </div>
        <div>
          <dt>Domínios na lista de bloqueio</dt>
          <dd>{{ fmtNumber(summary.domains_on_blocklist) }}</dd>
          <span v-if="summary.gravity_last_update" class="hint">
            Listas atualizadas {{ fmtRelative(summary.gravity_last_update) }}
          </span>
        </div>
      </dl>

      <h3>Mais bloqueados</h3>
      <p v-if="summary.queries_total === 0" class="muted empty">
        Nenhuma consulta nas últimas 24h. Confira se o roteador entrega o servidor como DNS.
      </p>
      <p v-else-if="summary.top_blocked.length === 0" class="muted empty">Nada bloqueado nas últimas 24h.</p>
      <ol v-else class="top">
        <li v-for="d in summary.top_blocked" :key="d.domain">
          <div class="top-row">
            <span class="domain" :title="d.domain">{{ d.domain }}</span>
            <span class="muted">{{ fmtNumber(d.count) }}</span>
          </div>
          <div class="bar">
            <span :style="{ width: `${(d.count / topMax) * 100}%` }" />
          </div>
        </li>
      </ol>
    </template>

    <footer class="foot">
      <p v-if="container" class="container muted">
        Container {{ fmtContainerStatus(container.state, container.status).toLowerCase() }}, CPU
        {{ fmtPercent(container.cpu_percent) }}, memória
        {{ container.mem_usage_mb === null ? '—' : fmtMb(container.mem_usage_mb) }}
      </p>
      <p v-else-if="container === null" class="container">
        <span class="led led-alert" /> Container do Pi-hole não encontrado
      </p>
      <div class="links">
        <a class="link-quiet" :href="httpsUrl(PIHOLE_SERVICE)" target="_blank" rel="noopener noreferrer">
          Abrir painel do Pi-hole
        </a>
        <a
          class="link-quiet"
          :href="directUrl(PIHOLE_SERVICE)"
          target="_blank"
          rel="noopener noreferrer"
          title="Acesso direto pela porta, sem passar pelo Caddy"
          >Direto: {{ directAddress(PIHOLE_SERVICE) }}</a
        >
      </div>
    </footer>
  </section>
</template>

<style scoped>
.pihole {
  padding: 1.2rem 1.3rem;
}
.state {
  display: inline-flex;
  align-items: center;
  gap: 0.45rem;
  font-weight: 500;
}
.figures {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 1rem 1.25rem;
  margin: 0;
}
dt {
  color: var(--engraved);
  font-size: 0.85rem;
}
dd {
  margin: 0.1rem 0 0;
  font-size: 1.6rem;
  font-weight: 600;
  line-height: 1.15;
}
.hint {
  color: var(--engraved);
  font-size: 0.85rem;
}
h3 {
  margin: 1.4rem 0 0.6rem;
  font-size: 1rem;
}
.empty {
  margin: 0;
  font-size: 0.92rem;
}
.top {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
  margin: 0;
  padding: 0;
  list-style: none;
}
.top-row {
  display: flex;
  justify-content: space-between;
  gap: 0.75rem;
  margin-bottom: 0.25rem;
  font-size: 0.92rem;
}
.domain {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.top .bar > span {
  background: var(--needle);
}
.foot {
  display: grid;
  gap: 0.4rem;
  margin-top: 1.3rem;
  padding-top: 0.9rem;
  border-top: 1px solid var(--rule);
}
.container {
  display: flex;
  align-items: center;
  gap: 0.45rem;
  margin: 0;
  font-size: 0.88rem;
}
.links {
  display: flex;
  flex-wrap: wrap;
  gap: 0.3rem 1.25rem;
}
</style>
