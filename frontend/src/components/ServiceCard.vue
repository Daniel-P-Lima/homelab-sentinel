<script setup lang="ts">
import { computed } from 'vue'
import type { ContainerInfo } from '../api'
import { fmtContainerStatus, fmtMb, fmtPercent } from '../format'
import { containerLed, ledClass, type Led } from '../leds'
import { directAddress, directUrl, httpsHost, httpsUrl, type Service } from '../services'

// Um canal do rack de serviços. As divisórias entre canais ficam no App.vue.
const props = defineProps<{
  service: Service
  /** undefined = o poll ainda não respondeu; null = container não existe no Docker */
  container: ContainerInfo | null | undefined
}>()

const status = computed<{ text: string; led: Led['state'] }>(() => {
  if (!props.service.container) return { text: 'Fora do Docker', led: 'off' }
  if (props.container === undefined) return { text: 'Lendo...', led: 'off' }
  if (props.container === null) return { text: 'Container não encontrado', led: 'alert' }
  const c = props.container
  return { text: fmtContainerStatus(c.state, c.status), led: containerLed(c.state, c.status) }
})
</script>

<template>
  <article class="channel">
    <header>
      <h3>
        <span class="led" :class="ledClass(status.led)" />
        {{ service.name }}
      </h3>
      <p class="muted desc">{{ service.description }}</p>
    </header>

    <p class="status">{{ status.text }}</p>

    <dl v-if="container" class="metrics">
      <div>
        <dt>CPU</dt>
        <dd>{{ fmtPercent(container.cpu_percent) }}</dd>
      </div>
      <div>
        <dt>Memória</dt>
        <dd>{{ container.mem_usage_mb === null ? '—' : fmtMb(container.mem_usage_mb) }}</dd>
      </div>
    </dl>

    <!-- noreferrer é obrigatório: o qBittorrent (proteção CSRF) responde
         401 "Unauthorized" a qualquer Referer de outra origem, como o do dashboard -->
    <footer class="actions">
      <a class="key" :href="httpsUrl(service)" target="_blank" rel="noopener noreferrer" :title="httpsHost(service)">
        Abrir {{ service.name }}
      </a>
      <a
        class="link-quiet"
        :href="directUrl(service)"
        target="_blank"
        rel="noopener noreferrer"
        title="Acesso direto pela porta, sem passar pelo Caddy"
        >Direto: {{ directAddress(service) }}</a
      >
    </footer>
  </article>
</template>

<style scoped>
.channel {
  display: flex;
  flex-direction: column;
  gap: 0.7rem;
  min-width: 0;
  padding: 1.1rem 1.25rem 1.2rem;
}
h3 {
  display: flex;
  align-items: center;
  gap: 0.55rem;
  font-size: 1.2rem;
}
.desc {
  margin: 0.15rem 0 0;
  font-size: 0.92rem;
}
.status {
  margin: 0;
  font-size: 0.95rem;
}
.metrics {
  display: flex;
  gap: 1.5rem;
  margin: 0;
}
dt {
  color: var(--engraved);
  font-size: 0.8rem;
}
dd {
  margin: 0;
  font-size: 1.05rem;
  font-weight: 600;
}
/* empurra as ações pro rodapé, alinhando canais de alturas diferentes */
.actions {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.45rem;
  margin-top: auto;
  padding-top: 0.3rem;
}
</style>
