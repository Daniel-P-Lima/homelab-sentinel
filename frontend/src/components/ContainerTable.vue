<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { fetchContainers, type ContainerInfo } from '../api'

const containers = ref<ContainerInfo[]>([])
const error = ref<string | null>(null)
const loading = ref(true)
let timer: number | undefined

async function poll() {
  try {
    containers.value = await fetchContainers()
    error.value = null
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'erro desconhecido'
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  poll()
  timer = window.setInterval(poll, 5000)
})

onUnmounted(() => {
  if (timer) window.clearInterval(timer)
})

function fmtPercent(v: number | null): string {
  return v === null ? '—' : `${v.toFixed(1)}%`
}

function fmtMem(usage: number | null, limit: number | null): string {
  if (usage === null) return '—'
  const usageStr = usage >= 1024 ? `${(usage / 1024).toFixed(1)} GB` : `${usage.toFixed(0)} MB`
  if (limit === null) return usageStr
  const limitStr = limit >= 1024 ? `${(limit / 1024).toFixed(1)} GB` : `${limit.toFixed(0)} MB`
  return `${usageStr} / ${limitStr}`
}
</script>

<template>
  <section>
    <h2>Containers</h2>

    <p v-if="loading">Carregando containers...</p>
    <p v-else-if="error" class="error">
      Não foi possível carregar os containers: {{ error }}
    </p>
    <p v-else-if="containers.length === 0">Nenhum container encontrado.</p>

    <table v-else>
      <thead>
        <tr>
          <th>Nome</th>
          <th>Imagem</th>
          <th>Estado</th>
          <th>CPU</th>
          <th>Memória</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="c in containers" :key="c.id" :class="`state-${c.state}`">
          <td>{{ c.name }}</td>
          <td class="image">{{ c.image }}</td>
          <td>
            <span class="badge" :class="`badge-${c.state}`">{{ c.status || c.state }}</span>
          </td>
          <td>{{ fmtPercent(c.cpu_percent) }}</td>
          <td>{{ fmtMem(c.mem_usage_mb, c.mem_limit_mb) }}</td>
        </tr>
      </tbody>
    </table>
  </section>
</template>

<style scoped>
table {
  width: 100%;
  border-collapse: collapse;
  margin-top: 0.5rem;
}
th, td {
  text-align: left;
  padding: 0.5rem 0.75rem;
  border-bottom: 1px solid #e0e0e0;
  font-size: 0.9rem;
}
.image {
  color: #666;
  font-family: monospace;
  font-size: 0.8rem;
}
.badge {
  padding: 0.15rem 0.5rem;
  border-radius: 999px;
  font-size: 0.75rem;
  background: #eee;
}
.badge-running {
  background: #d4edda;
  color: #1e7e34;
}
.badge-exited {
  background: #f8d7da;
  color: #a71d2a;
}
.error {
  color: #c0392b;
}
</style>
