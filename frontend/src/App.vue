<script setup lang="ts">
import { computed } from 'vue'
import { useContainers } from './composables/useContainers'
import { useHealth } from './composables/useHealth'
import { usePihole } from './composables/usePihole'
import type { Led } from './leds'
import { PIHOLE_SERVICE, SERVICES } from './services'
import ContainerTable from './components/ContainerTable.vue'
import HostCard from './components/HostCard.vue'
import PiholeCard from './components/PiholeCard.vue'
import ServiceCard from './components/ServiceCard.vue'

// Cada poll roda uma vez aqui e é repassado por props: o painel do servidor
// usa os três para os LEDs, e duplicar o de containers dobraria as stats.
const { health, error: healthError } = useHealth()
const { containers, others, containerOf, error: containersError, loading: containersLoading } = useContainers()
const { summary: pihole, error: piholeError, loading: piholeLoading } = usePihole()

const leds = computed<Led[]>(() => [
  healthError.value
    ? { label: 'Sentinel', state: 'alert', title: `Backend sem resposta: ${healthError.value}` }
    : { label: 'Sentinel', state: health.value ? 'on' : 'off', title: 'Backend do dashboard' },
  containersError.value
    ? { label: 'Docker', state: 'alert', title: containersError.value }
    : { label: 'Docker', state: containers.value ? 'on' : 'off', title: 'Leitura dos containers' },
  piholeError.value
    ? { label: 'DNS', state: 'alert', title: piholeError.value }
    : {
        label: 'DNS',
        state: !pihole.value ? 'off' : pihole.value.blocking === 'enabled' ? 'on' : 'warn',
        title: 'Pi-hole respondendo e bloqueando',
      },
])
</script>

<template>
  <div class="page">
    <p class="brand">
      <span>Homelab Sentinel</span>
      <span class="muted">Atualiza a cada 5 segundos</span>
    </p>

    <main>
      <HostCard :leds="leds" />

      <section class="rack" aria-label="Serviços">
        <ServiceCard
          v-for="s in SERVICES"
          :key="s.subdomain"
          :service="s"
          :container="containerOf(s.container)"
        />
      </section>

      <div class="lower">
        <PiholeCard
          :summary="pihole"
          :error="piholeError"
          :loading="piholeLoading"
          :container="containerOf(PIHOLE_SERVICE.container)"
        />
        <ContainerTable :containers="others" :error="containersError" :loading="containersLoading" />
      </div>
    </main>
  </div>
</template>

<style scoped>
.page {
  max-width: 1320px;
  margin: 0 auto;
  padding: 1.25rem 1rem 3rem;
}
.brand {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  gap: 0.25rem 1rem;
  margin: 0 0 0.9rem;
  font-weight: 500;
}
/* Os serviços num rack só: canais lado a lado, separados por divisórias,
   em vez de cartões soltos. No celular, empilham com divisória horizontal. */
.rack {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(min(100%, 230px), 1fr));
  margin: 1.25rem 0;
  border: 1px solid var(--panel-edge);
  border-radius: 6px;
  background: var(--panel);
  overflow: hidden;
}
/* Divisória à esquerda e acima de cada canal; as que caem na borda do rack
   ficam fora da área visível (overflow: hidden), então só sobram as internas,
   em qualquer número de colunas. */
.rack > * {
  box-shadow: -1px -1px 0 var(--rule);
}
.lower {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 1.25rem;
  align-items: start;
}
@media (min-width: 960px) {
  .lower {
    grid-template-columns: minmax(0, 2fr) minmax(0, 3fr);
  }
}
</style>
