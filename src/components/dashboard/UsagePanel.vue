<script setup lang="ts">
import type { UsagePoint } from '../../types/usage'

defineProps<{
  history: readonly UsagePoint[]
}>()

const _numberFormatter = new Intl.NumberFormat('es-ES', { notation: 'compact' })

function _barHeight(tokens: number, history: readonly UsagePoint[]) {
  const maxTokens = Math.max(...history.map((point) => point.tokens), 1)
  return `${Math.max(12, Math.round((tokens / maxTokens) * 100))}%`
}
</script>

<template>
  <section class="rounded-[1.6rem] border border-ledger-line bg-ledger-panel p-4" aria-labelledby="usage-title">
    <div class="flex items-center justify-between gap-3">
      <div>
        <p class="font-mono text-[0.65rem] uppercase tracking-[0.18em] text-ledger-muted">7 day ledger</p>
        <h2 id="usage-title" class="text-sm font-semibold text-ledger-ink">Weekly history</h2>
      </div>
      <span class="rounded-full bg-ledger-inset px-2.5 py-1 text-xs text-ledger-muted">tokens</span>
    </div>

    <div class="mt-3 flex h-24 items-end gap-2" role="img" aria-label="Bar chart of token usage over the last seven days">
      <div v-for="point in history" :key="point.day" class="flex min-w-0 flex-1 flex-col items-center gap-2">
        <div class="flex h-18 w-full items-end rounded-lg bg-ledger-inset p-1">
          <div class="w-full rounded-md bg-ledger-graphite" :style="{ height: barHeight(point.tokens, history) }"></div>
        </div>
        <div class="text-center">
          <p class="font-mono text-[0.65rem] text-ledger-ink">{{ numberFormatter.format(point.tokens) }}</p>
          <p class="text-[0.65rem] text-ledger-muted">{{ point.day }}</p>
        </div>
      </div>
    </div>
  </section>
</template>
