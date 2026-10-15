<script setup lang="ts">
import { computed } from 'vue'
import type { ContainerInfo } from '../api'
import { fmtMem, fmtPercent, memPercent, stateClass } from '../format'
import { hostAddress, serviceUrl } from '../links'

// Recebe os dados do App.vue em vez de fazer o próprio poll: os cards de
// serviço usam a mesma resposta (ver useContainers).
const props = defineProps<{
  /** Só os containers sem card próprio. */
  containers: ContainerInfo[] | null
  error: string | null
  loading: boolean
}>()

const runningCount = computed(() => props.containers?.filter((c) => c.state === 'running').length ?? 0)
</script>

<template>
  <section class="card">
    <header class="card-header">
      <h2>🐳 Outros containers</h2>
      <span v-if="containers" class="pill" :class="runningCount === containers.length ? 'pill-ok' : 'pill-warn'">
        {{ runningCount }} de {{ containers.length }} rodando
      </span>
    </header>

    <!-- erro aparece junto com a última tabela boa, não no lugar dela -->
    <p v-if="error" class="notice notice-danger">Não foi possível atualizar os containers: {{ error }}</p>
    <p v-if="loading" class="muted">Carregando containers... (a primeira leitura de CPU leva ~2s)</p>
    <p v-else-if="containers && containers.length === 0" class="muted">Nenhum container além dos serviços acima.</p>

    <div v-else-if="containers" class="table-wrap">
      <table>
        <thead>
          <tr>
            <th>Container</th>
            <th>Estado</th>
            <th class="num">CPU</th>
            <th>Memória</th>
            <th>Acesso</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="c in containers" :key="c.id">
            <td>
              <div class="name">{{ c.name }}</div>
              <div class="image" :title="c.image">{{ c.image }}</div>
            </td>
            <td>
              <span class="pill" :class="stateClass(c.state)">{{ c.status || c.state }}</span>
            </td>
            <td class="num">{{ fmtPercent(c.cpu_percent) }}</td>
            <td class="mem">
              <span>{{ fmtMem(c.mem_usage_mb, c.mem_limit_mb) }}</span>
              <div v-if="memPercent(c.mem_usage_mb, c.mem_limit_mb) !== null" class="bar">
                <span :style="{ width: `${memPercent(c.mem_usage_mb, c.mem_limit_mb)}%` }" />
              </div>
            </td>
            <td>
              <span v-if="c.ports.length === 0" class="muted">—</span>
              <div v-else class="ports">
                <template v-for="p in c.ports" :key="`${p.public_port}/${p.protocol}`">
                  <!-- só TCP vira link; UDP (ex: DNS na 53) o navegador não abre -->
                  <a
                    v-if="p.protocol === 'tcp'"
                    class="service-link"
                    :href="serviceUrl(p.public_port)"
                    target="_blank"
                    rel="noopener noreferrer"
                    :title="`porta ${p.private_port} do container`"
                    >{{ hostAddress(p.public_port) }}</a
                  >
                  <span v-else class="muted port-other" :title="`porta ${p.private_port} do container`">
                    {{ p.public_port }}/{{ p.protocol }}
                  </span>
                </template>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
</template>

<style scoped>
/* em tela estreita a tabela rola sozinha em vez de estourar a página */
.table-wrap {
  overflow-x: auto;
  margin: 0 -0.25rem;
}
table {
  width: 100%;
  border-collapse: collapse;
}
th,
td {
  text-align: left;
  padding: 0.65rem 0.5rem;
  border-bottom: 1px solid var(--border);
  font-size: 0.88rem;
  vertical-align: middle;
}
tbody tr:last-child td {
  border-bottom: none;
}
tbody tr:hover {
  background: var(--surface-2);
}
th {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.03em;
}
.num {
  text-align: right;
  font-variant-numeric: tabular-nums;
}
.name {
  font-weight: 600;
}
.image {
  max-width: 16rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-muted);
  font-family: ui-monospace, monospace;
  font-size: 0.75rem;
}
.mem {
  min-width: 9rem;
  font-variant-numeric: tabular-nums;
}
.ports {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
}
.port-other {
  font-family: ui-monospace, monospace;
  font-size: 0.78rem;
  white-space: nowrap;
}
.mem .bar {
  margin-top: 0.3rem;
}
</style>
