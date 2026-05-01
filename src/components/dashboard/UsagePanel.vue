<script setup lang="ts">
import type { UsagePoint } from "../../types/usage";

defineProps<{
  history: readonly UsagePoint[];
}>();

const numberFormatter = new Intl.NumberFormat("es-ES", { notation: "compact" });

function barHeight(tokens: number, history: readonly UsagePoint[]) {
  const maxTokens = Math.max(...history.map((point) => point.tokens), 1);
  return `${Math.max(12, Math.round((tokens / maxTokens) * 100))}%`;
}
</script>

<template>
  <section class="rounded-3xl border border-slate-800 bg-slate-900/60 p-5" aria-labelledby="usage-title">
    <div class="flex items-center justify-between gap-3">
      <div>
        <p class="font-mono text-xs uppercase tracking-[0.22em] text-slate-500">7 day ledger</p>
        <h2 id="usage-title" class="mt-1 text-xl font-semibold text-slate-50">Histórico semanal</h2>
      </div>
      <span class="rounded-full border border-blue-400/20 bg-blue-500/10 px-3 py-1 text-sm text-blue-100">tokens agregados</span>
    </div>

    <div class="mt-6 flex h-56 items-end gap-3" role="img" aria-label="Bar chart of token usage over the last seven days">
      <div v-for="point in history" :key="point.day" class="flex min-w-0 flex-1 flex-col items-center gap-3">
        <div class="flex h-44 w-full items-end rounded-xl bg-slate-950/70 p-1">
          <div class="w-full rounded-lg bg-gradient-to-t from-blue-600 to-amber-300" :style="{ height: barHeight(point.tokens, history) }"></div>
        </div>
        <div class="text-center">
          <p class="font-mono text-xs text-slate-300">{{ numberFormatter.format(point.tokens) }}</p>
          <p class="text-xs text-slate-500">{{ point.day }}</p>
        </div>
      </div>
    </div>
  </section>
</template>
