import { invoke } from "@tauri-apps/api/core";
import { computed, readonly, shallowRef } from "vue";
import type { DashboardSnapshot } from "../types/usage";

const emptySnapshot: DashboardSnapshot = {
  providers: [],
  history: [],
  syncEvents: [],
};

export function useDashboardData() {
  const snapshot = shallowRef<DashboardSnapshot>(emptySnapshot);
  const isLoading = shallowRef(false);
  const errorMessage = shallowRef<string | null>(null);

  const connectedProviders = computed(() =>
    snapshot.value.providers.filter((provider) => provider.status === "connected"),
  );

  const totalDailyTokens = computed(() =>
    snapshot.value.providers.reduce((sum, provider) => sum + provider.dailyTokens, 0),
  );

  const totalWeeklyTokens = computed(() =>
    snapshot.value.providers.reduce((sum, provider) => sum + provider.weeklyTokens, 0),
  );

  const totalCost = computed(() =>
    snapshot.value.providers.reduce((sum, provider) => sum + (provider.costUsd ?? 0), 0),
  );

  async function refresh() {
    isLoading.value = true;
    errorMessage.value = null;

    try {
      snapshot.value = await invoke<DashboardSnapshot>("get_dashboard_snapshot");
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : "No se pudo cargar el dashboard";
    } finally {
      isLoading.value = false;
    }
  }

  async function syncNow() {
    isLoading.value = true;
    errorMessage.value = null;

    try {
      snapshot.value = await invoke<DashboardSnapshot>("sync_all_providers");
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : "No se pudo sincronizar";
    } finally {
      isLoading.value = false;
    }
  }

  return {
    snapshot: readonly(snapshot),
    isLoading: readonly(isLoading),
    errorMessage: readonly(errorMessage),
    connectedProviders,
    totalDailyTokens,
    totalWeeklyTokens,
    totalCost,
    refresh,
    syncNow,
  };
}
