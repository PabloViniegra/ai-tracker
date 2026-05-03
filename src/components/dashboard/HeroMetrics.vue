<script setup lang="ts">
import { RefreshCw } from 'lucide-vue-next'

defineProps<{
  dailyTokens: number
  weeklyTokens: number
  totalCost: number
  connectedCount: number
  providerCount: number
  isLoading: boolean
}>()

defineEmits<{
  sync: []
}>()

const numberFormatter = new Intl.NumberFormat('es-ES')
const currencyFormatter = new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' })
</script>

<template>
  <section class="rounded-[1.8rem] border border-ledger-line bg-ledger-panel p-4" id="dashboard">
    <div class="flex items-center justify-between gap-3 px-1">
      <div>
        <p class="font-mono text-[0.65rem] uppercase tracking-[0.2em] text-ledger-muted">local usage ledger</p>
        <h1 class="text-lg font-semibold tracking-tight text-ledger-ink">AI Tracker</h1>
      </div>
      <button
        class="inline-flex min-h-9 items-center justify-center gap-2 rounded-full border border-ledger-line bg-ledger-inset px-3 text-xs font-semibold text-ledger-ink transition hover:bg-ledger-paper disabled:cursor-not-allowed disabled:opacity-60"
        :disabled="isLoading"
        type="button"
        @click="$emit('sync')"
      >
        <RefreshCw :class="['size-4', isLoading && 'animate-spin']" aria-hidden="true" />
        Sync now
      </button>
    </div>

    <div class="mt-3 grid gap-3 rounded-[1.4rem] border border-ledger-graphite bg-ledger-graphite p-4 text-ledger-paper sm:grid-cols-[minmax(0,1fr)_auto]">
      <div>
        <p class="font-mono text-[0.65rem] uppercase tracking-[0.18em] text-ledger-brass-soft">visible cost today</p>
        <strong class="font-mono text-3xl leading-none tracking-tight sm:text-[2.15rem]">{{ currencyFormatter.format(totalCost) }}</strong>
      </div>
      <div class="text-left text-xs leading-5 text-ledger-brass-soft sm:text-right">
        <strong class="block font-mono text-sm text-ledger-paper">{{ numberFormatter.format(dailyTokens) }} tokens</strong>
        {{ numberFormatter.format(weeklyTokens) }} this week
      </div>
      <div class="sm:col-span-2">
        <div class="grid h-2 grid-cols-[50%_35%_15%] overflow-hidden rounded-full bg-ledger-graphite-soft" aria-hidden="true">
          <span class="bg-ledger-brass"></span>
          <span class="bg-ledger-brass-soft"></span>
          <span class="bg-ledger-muted"></span>
        </div>
      </div>
    </div>

    <div class="mt-3 grid grid-cols-2 gap-2 text-xs">
      <p class="rounded-2xl bg-ledger-inset px-3 py-2">
        <strong class="block text-sm text-ledger-ink">{{ connectedCount }}/{{ providerCount }} active</strong>
        <span class="text-ledger-muted">provider coverage</span>
      </p>
      <p class="rounded-2xl bg-ledger-inset px-3 py-2">
        <strong class="block text-sm text-ledger-ink">Local vault</strong>
        <span class="text-ledger-muted">private credentials</span>
      </p>
    </div>
  </section>
</template>
