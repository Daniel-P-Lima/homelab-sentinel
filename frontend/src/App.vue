<script setup lang="ts">
import { useHealth } from './composables/useHealth'
import { fmtDuration } from './format'
import ContainerTable from './components/ContainerTable.vue'
import PiholeCard from './components/PiholeCard.vue'

const { health, error } = useHealth()
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

    <main class="grid">
      <PiholeCard />
      <ContainerTable />
    </main>
  </div>
</template>

<style scoped>
.page {
  max-width: 1180px;
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
