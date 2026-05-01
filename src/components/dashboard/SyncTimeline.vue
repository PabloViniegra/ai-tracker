<script setup lang="ts">
import { CircleAlert, CircleCheck, Info } from "lucide-vue-next";
import type { SyncEvent } from "../../types/usage";

defineProps<{
  events: readonly SyncEvent[];
}>();
</script>

<template>
  <section class="rounded-3xl border border-slate-800 bg-slate-900/60 p-5" aria-labelledby="sync-title">
    <p class="font-mono text-xs uppercase tracking-[0.22em] text-slate-500">Connector events</p>
    <h2 id="sync-title" class="mt-1 text-xl font-semibold text-slate-50">Sincronización</h2>

    <ol class="mt-5 space-y-3">
      <li v-for="event in events" :key="`${event.providerId}-${event.at}`" class="flex gap-3 rounded-2xl border border-slate-800 bg-slate-950/50 p-3">
        <CircleCheck v-if="event.status === 'success'" class="mt-0.5 text-emerald-300" :size="18" aria-hidden="true" />
        <CircleAlert v-else-if="event.status === 'error'" class="mt-0.5 text-red-300" :size="18" aria-hidden="true" />
        <Info v-else class="mt-0.5 text-amber-300" :size="18" aria-hidden="true" />
        <div class="min-w-0">
          <p class="text-sm font-semibold text-slate-100">{{ event.providerName }}</p>
          <p class="text-sm leading-6 text-slate-400">{{ event.message }}</p>
          <p class="mt-1 font-mono text-xs text-slate-600">{{ event.at }}</p>
        </div>
      </li>
    </ol>
  </section>
</template>
