<script setup lang="ts">
import { computed } from 'vue'
import type { ContainerInfo } from '../api'
import { fmtMem, fmtPercent, memPercent, stateClass } from '../format'
import { directAddress, directUrl, httpsHost, httpsUrl, type Service } from '../services'

const props = defineProps<{
  service: Service
  /** undefined = o poll ainda não respondeu; null = container não existe no Docker */
  container: ContainerInfo | null | undefined
}>()

const status = computed(() => {
  if (!props.service.container) return { text: 'Fora do Docker', cls: '' }
  if (props.container === undefined) return { text: 'Carregando...', cls: '' }
  if (props.container === null) return { text: 'Container não encontrado', cls: 'pill-danger' }
  return { text: props.container.status || props.container.state, cls: stateClass(props.container.state) }
})

const mem = computed(() => (props.container ? memPercent(props.container.mem_usage_mb, props.container.mem_limit_mb) : null))
</script>

<template>
  <section class="card service">
    <header class="service-header">
      <span class="icon" aria-hidden="true">{{ service.icon }}</span>
      <div class="title">
        <h2>{{ service.name }}</h2>
        <span class="muted desc">{{ service.description }}</span>
      </div>
    </header>

    <span class="pill" :class="status.cls" :title="status.text">{{ status.text }}</span>

    <dl v-if="container" class="metrics">
      <div>
        <dt>CPU</dt>
        <dd>{{ fmtPercent(container.cpu_percent) }}</dd>
      </div>
      <div>
        <dt>Memória</dt>
        <dd>{{ fmtMem(container.mem_usage_mb, null) }}</dd>
        <div v-if="mem !== null" class="bar" :title="`${mem.toFixed(0)}% do limite`">
          <span :style="{ width: `${mem}%` }" />
        </div>
      </div>
    </dl>

    <!-- noreferrer é obrigatório: o qBittorrent (proteção CSRF) responde
         401 "Unauthorized" a qualquer Referer de outra origem, como o do dashboard -->
    <div class="links">
      <a class="open" :href="httpsUrl(service)" target="_blank" rel="noopener noreferrer" :title="httpsUrl(service)">
        {{ httpsHost(service) }} ↗
      </a>
      <a class="service-link direct" :href="directUrl(service)" target="_blank" rel="noopener noreferrer" title="Acesso direto, sem o Caddy">
        direto: {{ directAddress(service) }}
      </a>
    </div>
  </section>
</template>

<style scoped>
.service {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.75rem;
  padding: 1rem;
}
.service-header {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  min-width: 0;
  width: 100%;
}
.icon {
  font-size: 1.5rem;
  line-height: 1;
}
.title {
  display: flex;
  flex-direction: column;
  min-width: 0;
}
h2 {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
}
.desc {
  font-size: 0.78rem;
}
.pill {
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
}
.metrics {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.75rem;
  width: 100%;
  margin: 0;
}
dt {
  font-size: 0.72rem;
  color: var(--text-muted);
}
dd {
  margin: 0.1rem 0 0.3rem;
  font-size: 0.95rem;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}
/* empurra os links pro rodapé, alinhando cards de alturas diferentes */
.links {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  width: 100%;
  margin-top: auto;
}
.open {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  padding: 0.45rem 0.65rem;
  border-radius: 8px;
  background: var(--surface-2);
  color: var(--accent);
  font-size: 0.82rem;
  font-weight: 500;
  text-decoration: none;
}
.open:hover {
  text-decoration: underline;
}
.direct {
  color: var(--text-muted);
  font-size: 0.72rem;
}
</style>
