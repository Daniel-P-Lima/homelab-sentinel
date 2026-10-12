<script setup lang="ts">
import { useHealth } from './composables/useHealth'
import ContainerTable from './components/ContainerTable.vue'

const { health, error } = useHealth()
</script>

<template>
  <main>
    <h1>Homelab Sentinel</h1>
    <p v-if="error" class="error">Backend indisponível: {{ error }}</p>
    <p v-else-if="health">
      Status: <strong>{{ health.status }}</strong>
      — uptime: {{ health.uptime_seconds }}s
    </p>
    <p v-else>Carregando...</p>

    <ContainerTable />
  </main>
</template>

<style scoped>
main {
  font-family: system-ui, sans-serif;
  max-width: 720px;
  margin: 4rem auto;
  padding: 0 1rem;
}
.error {
  color: #c0392b;
}
</style>
