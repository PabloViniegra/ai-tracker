<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useDashboardData } from './composables/useDashboardData'
import type { ProviderId } from './types/usage'

const dashboard = useDashboardData()

const _providerCount = computed(() => dashboard.snapshot.value.providers.length)

const activeModal = ref<{ providerId: ProviderId; setupType: 'openai' | 'anthropic' } | null>(null)

function _openModal(providerId: ProviderId) {
  const setupTypeMap: Record<string, 'openai' | 'anthropic'> = {
    openai: 'openai',
    anthropic: 'anthropic',
  }
  const setupType = setupTypeMap[providerId]
  if (setupType) {
    activeModal.value = { providerId, setupType }
  }
}

function _closeModal() {
  activeModal.value = null
}

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

        <ProviderGrid :providers="dashboard.snapshot.value.providers" @connect="openModal" />

        <div class="grid gap-3 md:grid-cols-[1fr_280px]">
          <UsagePanel :history="dashboard.snapshot.value.history" />
          <SyncTimeline :events="dashboard.snapshot.value.syncEvents" />
        </div>
      </main>
    </div>

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
