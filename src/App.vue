<script setup lang="ts">
import { computed, onMounted } from 'vue'
import HeroMetrics from './components/dashboard/HeroMetrics.vue'
import AnthropicSetupPanel from './components/dashboard/AnthropicSetupPanel.vue'
import GeminiSetupPanel from './components/dashboard/GeminiSetupPanel.vue'
import OpenAiSetupPanel from './components/dashboard/OpenAiSetupPanel.vue'
import ProviderGrid from './components/dashboard/ProviderGrid.vue'
import SyncTimeline from './components/dashboard/SyncTimeline.vue'
import UsagePanel from './components/dashboard/UsagePanel.vue'
import { useDashboardData } from './composables/useDashboardData'

const dashboard = useDashboardData()

const providerCount = computed(() => dashboard.snapshot.value.providers.length)

onMounted(() => {
  void dashboard.refresh()
})
</script>

<template>
  <div class="min-h-dvh bg-ledger-paper text-ledger-ink">
    <div class="mx-auto min-h-dvh w-full max-w-[760px] px-3 py-3 sm:px-4">
      <main class="flex flex-col gap-3" aria-label="AI usage dashboard">
        <HeroMetrics
          :daily-tokens="dashboard.totalDailyTokens.value"
          :weekly-tokens="dashboard.totalWeeklyTokens.value"
          :total-cost="dashboard.totalCost.value"
          :connected-count="dashboard.connectedProviders.value.length"
          :provider-count="providerCount"
          :is-loading="dashboard.isLoading.value"
          @sync="dashboard.syncNow"
        />

        <p v-if="dashboard.errorMessage.value" class="rounded-2xl border border-red-300 bg-red-50 px-4 py-3 text-sm text-red-800" role="alert">
          {{ dashboard.errorMessage.value }}
        </p>

        <ProviderGrid :providers="dashboard.snapshot.value.providers" />

        <div class="grid gap-3 md:grid-cols-[1fr_280px]">
          <UsagePanel :history="dashboard.snapshot.value.history" />
          <SyncTimeline :events="dashboard.snapshot.value.syncEvents" />
        </div>

        <div class="grid gap-3 md:grid-cols-2">
          <OpenAiSetupPanel @updated="dashboard.refresh" />
          <AnthropicSetupPanel @updated="dashboard.refresh" />
          <GeminiSetupPanel @updated="dashboard.refresh" />
        </div>
      </main>
    </div>
  </div>
</template>
