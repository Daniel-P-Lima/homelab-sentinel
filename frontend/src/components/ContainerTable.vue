<script setup lang="ts">
import { computed } from 'vue'
import type { ContainerInfo } from '../api'
import { fmtContainerStatus, fmtMem, fmtPercent, memPercent } from '../format'
import { containerLed, ledClass } from '../leds'
import { hostAddress, serviceUrl } from '../links'

// Recebe os dados do App.vue em vez de fazer o próprio poll: os canais de
// serviço usam a mesma resposta (ver useContainers).
const props = defineProps<{
  /** Só os containers sem canal próprio. */
  containers: ContainerInfo[] | null
  error: string | null
  loading: boolean
}>()

// Imagem recriada por `compose up --build` perde a tag e o Docker mostra o hash.
function fmtImage(image: string): string {
  return image.startsWith('sha256:') ? 'imagem local, sem tag' : image
}

const runningCount = computed(() => props.containers?.filter((c) => c.state === 'running').length ?? 0)
</script>

<template>
  <section class="display containers" aria-labelledby="containers-title">
    <header class="section-title">
      <h2 id="containers-title">Outros containers</h2>
      <span v-if="containers" class="muted">{{ runningCount }} de {{ containers.length }} no ar</span>
    </header>

    <!-- erro aparece junto com a última tabela boa, não no lugar dela -->
    <p v-if="error" class="notice notice-danger">Sem resposta do Docker: {{ error }}</p>
    <p v-if="loading" class="muted">Lendo os containers. A primeira leitura de CPU leva uns 2 segundos.</p>
    <p v-else-if="containers && containers.length === 0" class="muted">
      Todos os containers já aparecem nos canais acima.
    </p>

    <div v-else-if="containers" class="table-wrap">
      <table>
        <thead>
          <tr>
            <th scope="col">Container</th>
            <th scope="col">Estado</th>
            <th scope="col" class="num">CPU</th>
            <th scope="col">Memória</th>
            <th scope="col">Portas</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="c in containers" :key="c.id">
            <td>
              <div class="name">{{ c.name }}</div>
              <div class="image muted" :title="c.image">{{ fmtImage(c.image) }}</div>
            </td>
            <td>
              <span class="state">
                <span class="led" :class="ledClass(containerLed(c.state, c.status))" />
                {{ fmtContainerStatus(c.state, c.status) }}
              </span>
            </td>
            <td class="num">{{ fmtPercent(c.cpu_percent) }}</td>
            <td class="mem">
              <span>{{ fmtMem(c.mem_usage_mb, c.mem_limit_mb) }}</span>
              <div v-if="memPercent(c.mem_usage_mb, c.mem_limit_mb) !== null" class="bar">
                <span :style="{ width: `${memPercent(c.mem_usage_mb, c.mem_limit_mb)}%` }" />
              </div>
            </td>
            <td>
              <span v-if="c.ports.length === 0" class="muted" title="Nenhuma porta publicada no servidor">—</span>
              <ul v-else class="ports">
                <li v-for="p in c.ports" :key="`${p.public_port}/${p.protocol}`">
                  <!-- só TCP vira link; UDP (ex: DNS na 53) o navegador não abre -->
                  <a
                    v-if="p.protocol === 'tcp'"
                    class="link-quiet"
                    :href="serviceUrl(p.public_port)"
                    target="_blank"
                    rel="noopener noreferrer"
                    :title="`Porta ${p.private_port} dentro do container`"
                    >{{ hostAddress(p.public_port) }}</a
                  >
                  <span v-else class="muted" :title="`Porta ${p.private_port} dentro do container`">
                    {{ p.public_port }}/{{ p.protocol }}
                  </span>
                </li>
              </ul>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
</template>

<style scoped>
.containers {
  padding: 1.2rem 1.3rem;
}
/* em tela estreita a tabela rola sozinha em vez de estourar a página */
.table-wrap {
  overflow-x: auto;
  margin: 0 -0.4rem;
}
table {
  width: 100%;
  border-collapse: collapse;
}
th,
td {
  padding: 0.6rem 0.4rem;
  border-bottom: 1px solid var(--rule);
  text-align: left;
  vertical-align: top;
  font-size: 0.95rem;
}
tbody tr:last-child td {
  border-bottom: none;
}
th {
  color: var(--engraved);
  font-size: 0.85rem;
  font-weight: 500;
}
.num {
  text-align: right;
}
.name {
  font-weight: 600;
}
.image {
  max-width: 16rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 0.85rem;
}
.state {
  display: inline-flex;
  align-items: center;
  gap: 0.45rem;
  white-space: nowrap;
}
.mem {
  min-width: 9rem;
}
.mem .bar {
  margin-top: 0.35rem;
}
.ports {
  display: grid;
  gap: 0.15rem;
  margin: 0;
  padding: 0;
  list-style: none;
  white-space: nowrap;
}
</style>
