<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useDashboardData } from './composables/useDashboardData'
import { getDashboardShellStyle } from './lib/dashboardWindow'
import type { ProviderId } from './types/usage'

const dashboard = useDashboardData()
const dashboardShellStyle = getDashboardShellStyle()

const providerCount = computed(() => dashboard.snapshot.value.providers.length)

const activeModal = ref<{ providerId: ProviderId; setupType: 'openai' | 'anthropic' } | null>(null)

function openModal(providerId: ProviderId) {
  const setupTypeMap: Record<string, 'openai' | 'anthropic'> = {
    openai: 'openai',
    anthropic: 'anthropic',
  }
  const setupType = setupTypeMap[providerId]
  if (setupType) {
    activeModal.value = { providerId, setupType }
  }
}

function closeModal() {
  activeModal.value = null
}

onMounted(() => {
  void dashboard.refresh()
})
</script>

<template>
  <div class="flex h-dvh w-full items-stretch justify-center overflow-hidden bg-ledger-paper p-3 text-ledger-ink">
    <main
      class="grid h-full w-full grid-rows-[auto_auto_minmax(0,1fr)] gap-3 overflow-hidden rounded-[2rem] border border-ledger-line bg-ledger-paper/95 p-3 shadow-[0_20px_60px_-36px_rgba(39,34,24,0.45)]"
      :style="dashboardShellStyle"
      aria-label="AI usage dashboard"
    >
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

        <div class="grid min-h-0 gap-3 min-[920px]:grid-cols-[minmax(0,1.55fr)_320px]">
          <div class="min-h-0 overflow-hidden">
            <ProviderGrid :providers="dashboard.snapshot.value.providers" @connect="openModal" />
          </div>
          <div class="grid min-h-0 gap-3 min-[920px]:grid-rows-[auto_minmax(0,1fr)]">
            <UsagePanel :history="dashboard.snapshot.value.history" />
            <div class="min-h-0 overflow-hidden">
              <SyncTimeline :events="dashboard.snapshot.value.syncEvents" />
            </div>
          </div>
        </div>
      </main>

    <ProviderLoginModal
      v-if="activeModal"
      :provider-id="activeModal.providerId"
      :setup-type="activeModal.setupType"
      :visible="true"
      @close="closeModal"
    >
      <template v-if="activeModal.setupType === 'openai'">
        <OpenAiSetupPanel is-modal @updated="dashboard.refresh" />
      </template>
      <template v-else-if="activeModal.setupType === 'anthropic'">
        <AnthropicSetupPanel is-modal @updated="dashboard.refresh(); closeModal()" />
      </template>
    </ProviderLoginModal>
  </div>
</template>
