<script setup lang="ts">
import { Activity, Database, KeyRound, Settings, ShieldCheck } from "lucide-vue-next";

defineProps<{
  connectedCount: number;
  providerCount: number;
}>();
</script>

<template>
  <aside class="flex min-h-full flex-col border-r border-slate-800/80 bg-slate-950/90 px-5 py-6">
    <div class="flex items-center gap-3">
      <div class="grid size-11 place-items-center rounded-2xl border border-amber-400/30 bg-amber-400/10 text-amber-300">
        <Activity :size="22" aria-hidden="true" />
      </div>
      <div>
        <p class="font-mono text-xs uppercase tracking-[0.24em] text-slate-500">Local ledger</p>
        <h1 class="text-xl font-semibold tracking-tight text-slate-50">AI Tracker</h1>
      </div>
    </div>

    <nav class="mt-10 flex flex-col gap-2" aria-label="Main navigation">
      <a class="flex min-h-11 items-center gap-3 rounded-xl border border-blue-400/20 bg-blue-500/10 px-3 text-sm font-medium text-blue-100" href="#dashboard">
        <Database :size="18" aria-hidden="true" /> Dashboard
      </a>
      <a class="flex min-h-11 items-center gap-3 rounded-xl px-3 text-sm text-slate-400 transition hover:bg-slate-900 hover:text-slate-100" href="#providers">
        <KeyRound :size="18" aria-hidden="true" /> Providers
      </a>
      <a class="flex min-h-11 items-center gap-3 rounded-xl px-3 text-sm text-slate-400 transition hover:bg-slate-900 hover:text-slate-100" href="#settings">
        <Settings :size="18" aria-hidden="true" /> Settings
      </a>
    </nav>

    <div class="mt-auto rounded-2xl border border-emerald-400/20 bg-emerald-400/10 p-4">
      <div class="flex items-center gap-2 text-emerald-200">
        <ShieldCheck :size="18" aria-hidden="true" />
        <span class="text-sm font-semibold">Local-first</span>
      </div>
      <p class="mt-2 text-sm leading-6 text-emerald-100/70">
        {{ connectedCount }} de {{ providerCount }} conectores activos. Credenciales pensadas para vault local.
      </p>
    </div>
  </aside>
</template>
