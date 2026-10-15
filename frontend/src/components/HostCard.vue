<script setup lang="ts">
import { computed } from 'vue'
import { useHost } from '../composables/useHost'
import { fmtDecimal, fmtDuration, fmtMb } from '../format'
import { ledClass, type Led } from '../leds'
import { SERVER_NAME } from '../services'
import Meter from './Meter.vue'

// Os LEDs de Sentinel, Docker e DNS vêm do App.vue, que já faz esses polls.
defineProps<{ leds: Led[] }>()

const { host, error, loading } = useHost()

const BATTERY_STATUS: Record<string, string> = {
  Charging: 'carregando',
  Discharging: 'descarregando',
  Full: 'carregada',
  'Not charging': 'na tomada',
}

// O servidor é um notebook: bateria descarregando = faltou energia em casa.
const battery = computed(() => {
  const b = host.value?.battery
  if (!b) return null
  return { ...b, label: BATTERY_STATUS[b.status] ?? b.status.toLowerCase(), onBattery: b.status === 'Discharging' }
})

const powerLed = computed<Led>(() => {
  if (battery.value?.onBattery) return { label: 'Energia', state: 'alert', title: 'Sem energia na tomada: rodando na bateria' }
  if (host.value) return { label: 'Energia', state: 'on', title: 'Na tomada' }
  return { label: 'Energia', state: 'off', title: 'Sem leitura do servidor' }
})

const memPercent = computed(() => (host.value ? (host.value.mem_used_mb / host.value.mem_total_mb) * 100 : null))

function fmtGb(gb: number): string {
  return `${fmtDecimal(gb, gb >= 10 ? 0 : 1)} GB`
}

function fmtLoad(load: number[]): string {
  return load.map((l) => fmtDecimal(l, 2)).join('  ')
}
</script>

<template>
  <section class="front" aria-labelledby="server-name">
    <header class="front-top">
      <h1 id="server-name" class="badge">{{ SERVER_NAME }}</h1>
      <p v-if="host" class="uptime">Ligado há {{ fmtDuration(host.uptime_seconds) }}</p>
    </header>

    <div class="indicators">
      <ul class="leds">
        <li v-for="led in [powerLed, ...leds]" :key="led.label" :title="led.title">
          <span class="led" :class="ledClass(led.state)" />
          {{ led.label }}
        </li>
      </ul>

      <div v-if="battery" class="battery" :title="`Saúde: segura ${battery.health_percent?.toFixed(0) ?? '?'}% da carga de fábrica`">
        <span class="cells" :class="{ low: battery.onBattery }" aria-hidden="true">
          <span v-for="i in 10" :key="i" :class="{ full: i <= Math.round(battery.capacity_percent / 10) }" />
        </span>
        <span>Bateria {{ battery.capacity_percent }}%, {{ battery.label }}</span>
      </div>
    </div>

    <p v-if="battery?.onBattery" class="notice notice-danger">
      O servidor está na bateria ({{ battery.capacity_percent }}%). Confira se faltou energia ou se o carregador soltou.
    </p>
    <p v-if="error" class="notice notice-danger">Sem leitura do servidor: {{ error }}</p>

    <div class="meters" :aria-busy="loading">
      <Meter
        label="CPU"
        :value="host?.cpu_percent ?? null"
        :reading="host?.cpu_percent != null ? `${host.cpu_percent.toFixed(0)}%` : '—'"
        :detail="host ? `${host.cpu_count} núcleos, carga ${fmtLoad(host.load_avg)}` : ''"
      />
      <Meter
        label="Memória"
        :value="memPercent"
        :reading="host ? fmtMb(host.mem_used_mb) : '—'"
        :detail="host ? `de ${fmtMb(host.mem_total_mb)}, swap ${fmtMb(host.swap_used_mb)}` : ''"
      />
      <Meter
        label="Disco"
        :value="host?.disk?.used_percent ?? null"
        :reading="host?.disk ? `${host.disk.used_percent.toFixed(0)}%` : '—'"
        :detail="host?.disk ? `${fmtGb(host.disk.available_gb)} livres de ${fmtGb(host.disk.total_gb)}` : 'sem leitura do disco'"
        :danger="85"
      />
      <Meter
        label="Temperatura"
        :value="host?.temperature_c ?? null"
        :reading="host?.temperature_c != null ? `${host.temperature_c.toFixed(0)} °C` : '—'"
        detail="processador"
        :danger="80"
      />
    </div>
  </section>
</template>

<style scoped>
/* A frente de alumínio: o único lugar da página com textura e "parafusos". */
.front {
  position: relative;
  padding: 1.4rem 1.6rem 1.6rem;
  border: 1px solid var(--panel-edge);
  border-radius: 8px;
  background-color: var(--panel);
  background-image:
    radial-gradient(circle at 14px 14px, var(--panel-edge) 0 3px, transparent 3.5px),
    radial-gradient(circle at calc(100% - 14px) 14px, var(--panel-edge) 0 3px, transparent 3.5px),
    radial-gradient(circle at 14px calc(100% - 14px), var(--panel-edge) 0 3px, transparent 3.5px),
    radial-gradient(circle at calc(100% - 14px) calc(100% - 14px), var(--panel-edge) 0 3px, transparent 3.5px),
    var(--sheen),
    var(--brushed);
  box-shadow: 0 1px 0 rgb(255 255 255 / 0.4) inset, 0 6px 18px -10px rgb(0 0 0 / 0.35);
}
.front-top {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  justify-content: space-between;
  gap: 0.25rem 1rem;
}
/* nome do aparelho, como placa gravada */
.badge {
  font-size: clamp(1.9rem, 5vw, 2.8rem);
  font-weight: 600;
  letter-spacing: -0.01em;
  color: var(--ink);
  text-shadow: 0 1px 0 rgb(255 255 255 / 0.35);
}
.uptime {
  margin: 0;
  color: var(--engraved);
}
.indicators {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem 1.5rem;
  margin: 0.9rem 0 1.25rem;
  padding: 0.7rem 0;
  border-block: 1px solid var(--rule);
}
.leds {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem 1.4rem;
  margin: 0;
  padding: 0;
  list-style: none;
}
.leds li {
  display: inline-flex;
  align-items: center;
  gap: 0.45rem;
  font-weight: 500;
}
.battery {
  display: inline-flex;
  align-items: center;
  gap: 0.6rem;
  color: var(--engraved);
}
.cells {
  display: inline-flex;
  gap: 2px;
  padding: 2px;
  border: 1px solid var(--engraved);
  border-radius: 3px;
}
.cells span {
  width: 5px;
  height: 11px;
  border-radius: 1px;
  background: var(--rule);
}
.cells span.full {
  background: var(--led-on);
}
.cells.low span.full {
  background: var(--led-alert);
}
.meters {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(min(100%, 160px), 1fr));
  gap: 1.25rem 1.5rem;
}
@media (max-width: 520px) {
  .front {
    padding: 1.2rem 1rem 1.3rem;
  }
  .meters {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
