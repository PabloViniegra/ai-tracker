<script setup lang="ts">
import { CircleAlert, CircleCheck, Info } from "lucide-vue-next";
import type { SyncEvent } from "../../types/usage";

defineProps<{
  events: readonly SyncEvent[];
}>();
</script>

<template>
  <section class="rounded-[1.5rem] border border-ledger-line bg-ledger-panel p-4" aria-labelledby="sync-title">
    <p class="font-mono text-[0.65rem] uppercase tracking-[0.18em] text-ledger-muted">Connector events</p>
    <h2 id="sync-title" class="text-sm font-semibold text-ledger-ink">Sync log</h2>

    <ol class="mt-4 space-y-2">
      <li v-for="event in events" :key="`${event.providerId}-${event.at}`" class="flex gap-2 rounded-2xl bg-ledger-inset p-3">
        <CircleCheck v-if="event.status === 'success'" class="mt-0.5 text-emerald-700" :size="16" aria-hidden="true" />
        <CircleAlert v-else-if="event.status === 'error'" class="mt-0.5 text-red-700" :size="16" aria-hidden="true" />
        <Info v-else class="mt-0.5 text-amber-700" :size="16" aria-hidden="true" />
        <div class="min-w-0">
          <p class="text-xs font-semibold text-ledger-ink">{{ event.providerName }}</p>
          <p class="text-xs leading-5 text-ledger-muted">{{ event.message }}</p>
          <p class="mt-1 font-mono text-[0.65rem] text-ledger-muted">{{ event.at }}</p>
        </div>
      </li>
    </ol>
  </section>
</template>
