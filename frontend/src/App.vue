<script setup lang="ts">
import { useContainers } from './composables/useContainers'
import { useHealth } from './composables/useHealth'
import { fmtDuration } from './format'
import { PIHOLE_SERVICE, SERVICES } from './services'
import ContainerTable from './components/ContainerTable.vue'
import PiholeCard from './components/PiholeCard.vue'
import ServiceCard from './components/ServiceCard.vue'

const { health, error } = useHealth()
const { others, containerOf, error: containersError, loading: containersLoading } = useContainers()
</script>

<template>
  <div class="page">
    <header class="top">
      <div>
        <h1>Homelab Sentinel</h1>
        <p class="muted subtitle">Tudo do homelab numa tela só · atualiza a cada 5s</p>
      </div>
      <span v-if="error" class="pill pill-danger" :title="error">Backend offline</span>
      <span v-else-if="health" class="pill pill-ok">Online · há {{ fmtDuration(health.uptime_seconds) }}</span>
      <span v-else class="pill">Conectando...</span>
    </header>

    <main>
      <section class="services" aria-label="Serviços">
        <ServiceCard
          v-for="s in SERVICES"
          :key="s.subdomain"
          :service="s"
          :container="containerOf(s.container)"
        />
      </section>

      <div class="grid">
        <PiholeCard :container="containerOf(PIHOLE_SERVICE.container)" />
        <ContainerTable :containers="others" :error="containersError" :loading="containersLoading" />
      </div>
    </main>
  </div>
</template>

<style scoped>
.page {
  max-width: 1440px;
  margin: 0 auto;
  padding: 2rem 1rem 3rem;
}
.top {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  margin-bottom: 1.5rem;
}
h1 {
  margin: 0;
  font-size: 1.6rem;
  font-weight: 700;
}
.subtitle {
  margin: 0.25rem 0 0;
  font-size: 0.9rem;
}
/* os 4 serviços numa linha só a partir de ~1000px de largura; abaixo disso
   quebra, e no celular fica um por linha. auto-fit estica os cards pra
   ocupar a linha toda em vez de deixar colunas vazias. */
.services {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(min(100%, 220px), 1fr));
  gap: 1rem;
  margin-bottom: 1.25rem;
}
/* Pi-hole na coluna estreita, containers na larga; em tela pequena empilha */
.grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 1.25rem;
  align-items: start;
}
@media (min-width: 900px) {
  .grid {
    grid-template-columns: minmax(0, 2fr) minmax(0, 3fr);
  }
}
</style>
