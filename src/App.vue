<script setup lang="ts">
import { computed, onMounted } from "vue";
import AppSidebar from "./components/dashboard/AppSidebar.vue";
import HeroMetrics from "./components/dashboard/HeroMetrics.vue";
import AnthropicSetupPanel from "./components/dashboard/AnthropicSetupPanel.vue";
import OpenAiSetupPanel from "./components/dashboard/OpenAiSetupPanel.vue";
import ProviderGrid from "./components/dashboard/ProviderGrid.vue";
import SyncTimeline from "./components/dashboard/SyncTimeline.vue";
import UsagePanel from "./components/dashboard/UsagePanel.vue";
import { useDashboardData } from "./composables/useDashboardData";

const dashboard = useDashboardData();

const providerCount = computed(() => dashboard.snapshot.value.providers.length);

onMounted(() => {
  void dashboard.refresh();
});
</script>

<template>
  <div class="min-h-dvh bg-slate-950 text-slate-100">
    <div class="grid min-h-dvh lg:grid-cols-[280px_1fr]">
      <AppSidebar :connected-count="dashboard.connectedProviders.value.length" :provider-count="providerCount" />

      <main class="min-w-0 px-4 py-4 md:px-6 lg:px-8" aria-label="AI usage dashboard">
        <div class="mx-auto flex max-w-7xl flex-col gap-5">
          <HeroMetrics
            :daily-tokens="dashboard.totalDailyTokens.value"
            :weekly-tokens="dashboard.totalWeeklyTokens.value"
            :total-cost="dashboard.totalCost.value"
            :is-loading="dashboard.isLoading.value"
            @sync="dashboard.syncNow"
          />

          <p v-if="dashboard.errorMessage.value" class="rounded-2xl border border-red-400/30 bg-red-400/10 px-4 py-3 text-sm text-red-100" role="alert">
            {{ dashboard.errorMessage.value }}
          </p>

          <ProviderGrid :providers="dashboard.snapshot.value.providers" />

          <div class="grid gap-5 xl:grid-cols-[1fr_380px]">
            <UsagePanel :history="dashboard.snapshot.value.history" />
            <SyncTimeline :events="dashboard.snapshot.value.syncEvents" />
          </div>

          <OpenAiSetupPanel @updated="dashboard.refresh" />
          <AnthropicSetupPanel @updated="dashboard.refresh" />
        </div>
      </main>
    </div>
  </div>
</template>
