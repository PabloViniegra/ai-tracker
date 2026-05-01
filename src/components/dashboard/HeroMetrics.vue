<script setup lang="ts">
import { Clock3, DollarSign, Gauge, RefreshCw } from "lucide-vue-next";

defineProps<{
  dailyTokens: number;
  weeklyTokens: number;
  totalCost: number;
  isLoading: boolean;
}>();

defineEmits<{
  sync: [];
}>();

const numberFormatter = new Intl.NumberFormat("es-ES");
const currencyFormatter = new Intl.NumberFormat("en-US", { style: "currency", currency: "USD" });
</script>

<template>
  <section class="rounded-[2rem] border border-slate-800 bg-slate-900/70 p-5 shadow-2xl shadow-blue-950/20" id="dashboard">
    <div class="flex flex-col gap-5 lg:flex-row lg:items-end lg:justify-between">
      <div>
        <p class="font-mono text-xs uppercase tracking-[0.28em] text-amber-300">Subscription telemetry</p>
        <h2 class="mt-3 max-w-3xl text-3xl font-semibold tracking-tight text-slate-50 md:text-5xl">
          Control diario de tokens sin sacar credenciales del equipo.
        </h2>
      </div>
      <button
        class="inline-flex min-h-11 items-center justify-center gap-2 rounded-xl bg-amber-500 px-4 text-sm font-semibold text-slate-950 transition hover:bg-amber-400 disabled:cursor-not-allowed disabled:opacity-60"
        :disabled="isLoading"
        type="button"
        @click="$emit('sync')"
      >
        <RefreshCw :class="['size-4', isLoading && 'animate-spin']" aria-hidden="true" />
        Actualizar ahora
      </button>
    </div>

    <div class="mt-6 grid gap-3 md:grid-cols-3">
      <article class="rounded-2xl border border-blue-400/20 bg-blue-500/10 p-4">
        <Gauge class="text-blue-200" :size="20" aria-hidden="true" />
        <p class="mt-4 text-sm text-blue-100/70">Tokens hoy</p>
        <strong class="font-mono text-3xl text-blue-50">{{ numberFormatter.format(dailyTokens) }}</strong>
      </article>
      <article class="rounded-2xl border border-slate-700 bg-slate-950/60 p-4">
        <Clock3 class="text-slate-300" :size="20" aria-hidden="true" />
        <p class="mt-4 text-sm text-slate-400">Tokens semana</p>
        <strong class="font-mono text-3xl text-slate-50">{{ numberFormatter.format(weeklyTokens) }}</strong>
      </article>
      <article class="rounded-2xl border border-amber-400/20 bg-amber-400/10 p-4">
        <DollarSign class="text-amber-200" :size="20" aria-hidden="true" />
        <p class="mt-4 text-sm text-amber-100/70">Coste visible</p>
        <strong class="font-mono text-3xl text-amber-50">{{ currencyFormatter.format(totalCost) }}</strong>
      </article>
    </div>
  </section>
</template>
