<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { fetchHealth, type HealthResponse } from './api'

const health = ref<HealthResponse | null>(null)
const error = ref<string | null>(null)
let timer: number | undefined

async function poll() {
  try {
    health.value = await fetchHealth()
    error.value = null
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'erro desconhecido'
  }
}

onMounted(() => {
  poll()
  timer = window.setInterval(poll, 5000)
})

onUnmounted(() => {
  if (timer) window.clearInterval(timer)
})
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

    <!--
      Fase 1 vai adicionar <ContainerTable /> aqui, consumindo
      GET /api/containers. Ver docs/02-Roadmap.md.
    -->
  </main>
</template>

<style scoped>
main {
  font-family: system-ui, sans-serif;
  max-width: 640px;
  margin: 4rem auto;
  padding: 0 1rem;
}
.error {
  color: #c0392b;
}
</style>
