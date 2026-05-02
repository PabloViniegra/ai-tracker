<script setup lang="ts">
import type { ProviderSummary } from '../../types/usage'

defineProps<{
  providers: readonly ProviderSummary[]
}>()

const _numberFormatter = new Intl.NumberFormat('es-ES')

function _statusLabel(status: ProviderSummary['status']) {
  const labels = {
    connected: 'Conectado',
    needs_credentials: 'Credenciales',
    experimental: 'Experimental',
    unsupported: 'Limitado',
  } satisfies Record<ProviderSummary['status'], string>

  return labels[status]
}

function _sourceLabel(source: ProviderSummary['source']) {
  const labels = {
    official_api: 'API oficial',
    local_estimate: 'Estimado local',
    manual: 'Manual',
  } satisfies Record<ProviderSummary['source'], string>

  return labels[source]
}
</script>

<template>
  <section class="grid gap-4 lg:grid-cols-3" id="providers" aria-label="Provider connectors">
    <article v-for="provider in providers" :key="provider.id" class="rounded-3xl border border-slate-800 bg-slate-900/60 p-4 transition hover:border-blue-400/40">
      <div class="flex items-start justify-between gap-3">
        <div>
          <p class="font-mono text-xs uppercase tracking-[0.2em] text-slate-500">{{ provider.id }}</p>
          <h3 class="mt-1 text-lg font-semibold text-slate-50">{{ provider.name }}</h3>
        </div>
        <span class="inline-flex items-center gap-1 rounded-full border px-2.5 py-1 text-xs font-medium" :class="{
          'border-emerald-400/30 bg-emerald-400/10 text-emerald-200': provider.status === 'connected',
          'border-amber-400/30 bg-amber-400/10 text-amber-200': provider.status === 'needs_credentials',
          'border-purple-400/30 bg-purple-400/10 text-purple-200': provider.status === 'experimental',
          'border-slate-600 bg-slate-800 text-slate-300': provider.status === 'unsupported'
        }">
          <CheckCircle2 v-if="provider.status === 'connected'" :size="14" aria-hidden="true" />
          <LockKeyhole v-else-if="provider.status === 'needs_credentials'" :size="14" aria-hidden="true" />
          <FlaskConical v-else-if="provider.status === 'experimental'" :size="14" aria-hidden="true" />
          <AlertTriangle v-else :size="14" aria-hidden="true" />
          {{ statusLabel(provider.status) }}
        </span>
      </div>

      <div class="mt-5 flex items-end justify-between gap-3">
        <div>
          <p class="text-sm text-slate-400">Hoy</p>
          <p class="font-mono text-2xl font-semibold text-slate-50">{{ numberFormatter.format(provider.dailyTokens) }}</p>
        </div>
        <div class="text-right">
          <p class="text-sm text-slate-400">Semana</p>
          <p class="font-mono text-lg text-slate-200">{{ numberFormatter.format(provider.weeklyTokens) }}</p>
        </div>
      </div>

      <div class="mt-4 h-2 overflow-hidden rounded-full bg-slate-800" aria-hidden="true">
        <div class="h-full rounded-full bg-gradient-to-r from-blue-500 to-amber-400" :style="{ width: `${Math.min(100, Math.max(8, provider.quotaUsed ?? 24))}%` }"></div>
      </div>

      <div class="mt-4 flex flex-wrap gap-2 text-xs">
        <span class="rounded-full bg-slate-800 px-2.5 py-1 text-slate-300">{{ sourceLabel(provider.source) }}</span>
        <span class="rounded-full bg-slate-800 px-2.5 py-1 text-slate-300">Confianza {{ provider.confidence }}</span>
        <span v-if="provider.capabilities.cost" class="rounded-full bg-slate-800 px-2.5 py-1 text-slate-300">Coste</span>
        <span v-if="provider.capabilities.quota" class="rounded-full bg-slate-800 px-2.5 py-1 text-slate-300">Cuota</span>
      </div>
    </article>
  </section>
</template>
