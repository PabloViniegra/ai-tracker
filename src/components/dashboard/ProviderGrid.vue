<script setup lang="ts">
import { clampUsagePercent, getProviderLogo, statusLabel } from '../../lib/providerPresentation'
import type { ProviderId, ProviderSummary } from '../../types/usage'

defineProps<{
  providers: readonly ProviderSummary[]
}>()

const emit = defineEmits<{
  connect: [providerId: ProviderId]
}>()

const numberFormatter = new Intl.NumberFormat('es-ES')
const currencyFormatter = new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' })
</script>

<template>
  <section class="flex h-full min-h-0 flex-col rounded-[1.8rem] border border-ledger-line bg-ledger-panel p-4" id="providers" aria-label="Provider connectors">
    <div class="flex items-center justify-between px-1 pb-2">
      <h2 class="text-sm font-semibold text-ledger-ink">Provider cost strips</h2>
      <span class="font-mono text-[0.65rem] uppercase tracking-[0.16em] text-ledger-muted">tokens · cost · source</span>
    </div>

    <div class="min-h-0 divide-y divide-ledger-line/70 overflow-y-auto pr-1">
      <article v-for="provider in providers" :key="provider.id" class="grid gap-2 py-3 first:pt-2 sm:grid-cols-[150px_1fr_auto] sm:items-center">
        <div class="flex min-w-0 items-center gap-3">
          <span class="grid size-9 shrink-0 place-items-center rounded-xl border border-ledger-line bg-ledger-paper text-[0.68rem] font-bold text-ledger-ink">
            <img v-if="getProviderLogo(provider.id).src" class="size-5" :src="getProviderLogo(provider.id).src ?? ''" :alt="`${getProviderLogo(provider.id).label} logo`" />
            <span v-else aria-hidden="true">{{ getProviderLogo(provider.id).initials }}</span>
          </span>
          <span class="min-w-0">
            <strong class="block truncate text-sm text-ledger-ink">{{ provider.name }}</strong>
            <span class="font-mono text-[0.65rem] uppercase tracking-[0.12em] text-ledger-muted">{{ statusLabel(provider.status) }}</span>
          </span>
        </div>

        <div class="grid gap-1">
          <div class="flex items-center justify-between gap-3 text-xs text-ledger-muted">
            <span>{{ numberFormatter.format(provider.dailyTokens) }} today</span>
            <span class="font-mono text-ledger-ink">{{ provider.costUsd == null ? 'cost n/a' : currencyFormatter.format(provider.costUsd) }}</span>
          </div>
          <div class="h-1.5 overflow-hidden rounded-full bg-ledger-inset" :aria-label="`${provider.name}: ${clampUsagePercent(provider.quotaUsed)}% shown`">
            <div class="h-full rounded-full bg-ledger-graphite" :style="{ width: `${clampUsagePercent(provider.quotaUsed)}%` }"></div>
          </div>
        </div>

        <div class="flex flex-wrap items-center gap-1.5">
          <button
            v-if="provider.status === 'needs_credentials'"
            class="inline-flex items-center gap-1.5 rounded-full bg-ledger-brass px-3 py-1.5 text-xs font-semibold text-ledger-paper transition hover:bg-ledger-brass-soft"
            type="button"
            :aria-label="`Connect to ${provider.name}`"
            @click="emit('connect', provider.id)"
          >
            <KeyRound :size="14" aria-hidden="true" /> Connect
          </button>
          <SourceIndicator
            :source="provider.source"
            :confidence="provider.confidence"
            :last-sync="provider.lastSync"
          />
        </div>
      </article>
    </div>
  </section>
</template>
