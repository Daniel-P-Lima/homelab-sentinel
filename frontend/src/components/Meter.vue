<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'

// Medidor de ponteiro, no estilo VU-meter: escala de 0 a 100 num arco de 120°,
// com a faixa de alerta em vermelho a partir de `danger`.
const props = withDefaults(
  defineProps<{
    label: string
    /** 0–100; null deixa o ponteiro em repouso e o mostrador apagado. */
    value: number | null
    reading: string
    detail?: string
    danger?: number
  }>(),
  { detail: '', danger: 90 },
)

const SWEEP = 120
const CX = 60
const CY = 62
const R = 48

function angle(v: number): number {
  return -SWEEP / 2 + (Math.min(Math.max(v, 0), 100) / 100) * SWEEP
}

function point(deg: number, r: number): [number, number] {
  const rad = ((deg - 90) * Math.PI) / 180
  return [CX + r * Math.cos(rad), CY + r * Math.sin(rad)]
}

function arc(from: number, to: number, r: number): string {
  const [x1, y1] = point(angle(from), r)
  const [x2, y2] = point(angle(to), r)
  return `M ${x1.toFixed(2)} ${y1.toFixed(2)} A ${r} ${r} 0 0 1 ${x2.toFixed(2)} ${y2.toFixed(2)}`
}

const ticks = Array.from({ length: 11 }, (_, i) => {
  const deg = angle(i * 10)
  const major = i % 5 === 0
  const [x1, y1] = point(deg, R)
  const [x2, y2] = point(deg, R - (major ? 8 : 4.5))
  return { x1, y1, x2, y2, major }
})

// O ponteiro sai do zero na primeira leitura: o único movimento de entrada da página.
const mounted = ref(false)
onMounted(() => requestAnimationFrame(() => (mounted.value = true)))

const needle = computed(() => angle(mounted.value && props.value !== null ? props.value : 0))
const inDanger = computed(() => props.value !== null && props.value >= props.danger)
</script>

<template>
  <figure class="meter" :class="{ off: value === null, hot: inDanger }">
    <svg viewBox="0 0 120 70" role="img" :aria-label="`${label}: ${reading}`">
      <path class="scale" :d="arc(0, danger, R)" />
      <path class="scale-danger" :d="arc(danger, 100, R)" />
      <line
        v-for="(t, i) in ticks"
        :key="i"
        :class="t.major ? 'tick major' : 'tick'"
        :x1="t.x1"
        :y1="t.y1"
        :x2="t.x2"
        :y2="t.y2"
      />
      <g class="needle" :style="{ transform: `rotate(${needle}deg)` }">
        <line :x1="CX" :y1="CY" :x2="CX" :y2="CY - R + 2" />
      </g>
      <circle class="pivot" :cx="CX" :cy="CY" r="3.2" />
    </svg>
    <figcaption>
      <span class="label">{{ label }}</span>
      <span class="reading">{{ reading }}</span>
      <span v-if="detail" class="detail">{{ detail }}</span>
    </figcaption>
  </figure>
</template>

<style scoped>
.meter {
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}
svg {
  display: block;
  width: 100%;
  max-width: 260px;
  padding: 0.35rem 0.5rem 0;
  background: var(--display);
  border: 1px solid var(--rule);
  border-radius: 6px 6px 3px 3px;
  box-shadow: inset 0 2px 4px rgb(0 0 0 / 0.1);
}
.scale,
.scale-danger {
  fill: none;
  stroke-width: 1.2;
}
.scale {
  stroke: var(--ink);
}
.scale-danger {
  stroke: var(--needle);
  stroke-width: 3;
}
.tick {
  stroke: var(--ink);
  stroke-width: 0.8;
}
.tick.major {
  stroke-width: 1.4;
}
.needle {
  transform-origin: 60px 62px;
  transition: transform 0.7s cubic-bezier(0.3, 1.35, 0.5, 1);
}
.needle line {
  stroke: var(--needle);
  stroke-width: 1.6;
  stroke-linecap: round;
}
.pivot {
  fill: var(--ink);
}
.off .needle line {
  stroke: var(--led-off);
}
figcaption {
  display: grid;
  gap: 0.05rem;
}
.label {
  color: var(--engraved);
  font-size: 0.85rem;
  font-weight: 500;
  letter-spacing: 0.02em;
}
.reading {
  font-size: 1.55rem;
  font-weight: 600;
  line-height: 1.1;
}
.hot .reading {
  color: var(--led-alert);
}
.detail {
  color: var(--engraved);
  font-size: 0.85rem;
}
</style>
