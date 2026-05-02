<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { KeyRound, RefreshCw, Shield } from "lucide-vue-next";
import { onMounted, reactive, shallowRef } from "vue";
import type {
  DashboardSnapshot,
  OpenAiConnectionState,
  SaveOpenAiCredentialsInput,
  SaveOpenAiCredentialsResult,
} from "../../types/usage";

const emit = defineEmits<{
  updated: [];
}>();

const form = reactive({
  apiKey: "",
  accountLabel: "",
  organizationId: "",
  projectId: "",
});

const connection = shallowRef<OpenAiConnectionState | null>(null);
const isLoading = shallowRef(false);
const message = shallowRef<string | null>(null);
const errorMessage = shallowRef<string | null>(null);

async function loadConnection() {
  try {
    connection.value = await invoke<OpenAiConnectionState>("get_openai_connection");
    form.accountLabel = connection.value.accountLabel ?? "";
    form.organizationId = connection.value.organizationId ?? "";
    form.projectId = connection.value.projectId ?? "";
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : "Could not read OpenAI configuration";
  }
}

async function saveCredentials() {
  isLoading.value = true;
  message.value = null;
  errorMessage.value = null;

  try {
    const payload: SaveOpenAiCredentialsInput = {
      apiKey: form.apiKey,
      accountLabel: form.accountLabel || null,
      organizationId: form.organizationId || null,
      projectId: form.projectId || null,
    };

    const result = await invoke<SaveOpenAiCredentialsResult>("save_openai_credentials", { input: payload });
    connection.value = result.connection;
    message.value = result.message;
    form.apiKey = "";
    emit("updated");
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : "Could not save the API key";
  } finally {
    isLoading.value = false;
  }
}

async function syncOpenAi() {
  isLoading.value = true;
  message.value = null;
  errorMessage.value = null;

  try {
    await invoke<DashboardSnapshot>("sync_all_providers");
    await loadConnection();
    message.value = "Sync requested. Check the sync log for OpenAI usage results or permission warnings.";
    emit("updated");
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : "Could not sync OpenAI";
  } finally {
    isLoading.value = false;
  }
}

onMounted(() => {
  void loadConnection();
});
</script>

<template>
  <section class="rounded-[1.5rem] border border-ledger-line bg-ledger-panel p-4" id="settings" aria-labelledby="openai-title">
    <div class="flex items-start justify-between gap-3">
      <div>
        <p class="font-mono text-[0.65rem] uppercase tracking-[0.18em] text-ledger-muted">Vault + SQLite</p>
        <h2 id="openai-title" class="text-sm font-semibold text-ledger-ink">Connect OpenAI</h2>
      </div>
      <span class="inline-flex items-center gap-2 rounded-full bg-ledger-inset px-2.5 py-1 text-xs text-ledger-muted">
        <Shield :size="14" aria-hidden="true" /> keyring local
      </span>
    </div>

    <p class="mt-3 text-xs leading-5 text-ledger-muted">
      The API key is stored in the system keyring. Metadata and history remain in local SQLite.
      Real usage requires an <span class="font-semibold text-ledger-ink">Admin Key</span> for OpenAI usage/costs endpoints.
    </p>

    <form class="mt-5 grid gap-3" @submit.prevent="saveCredentials">
      <label class="grid gap-2">
        <span class="text-xs text-ledger-muted">API key</span>
        <input v-model="form.apiKey" class="min-h-10 rounded-xl border border-ledger-line bg-ledger-paper px-3 text-sm text-ledger-ink outline-none transition focus:border-ledger-brass" placeholder="sk-..." type="password" autocomplete="off" />
      </label>

      <div class="grid gap-3 md:grid-cols-2">
        <label class="grid gap-2">
          <span class="text-xs text-ledger-muted">Account label</span>
          <input v-model="form.accountLabel" class="min-h-10 rounded-xl border border-ledger-line bg-ledger-paper px-3 text-sm text-ledger-ink outline-none transition focus:border-ledger-brass" placeholder="Personal / Team" type="text" />
        </label>
        <label class="grid gap-2">
          <span class="text-xs text-ledger-muted">Organization ID</span>
          <input v-model="form.organizationId" class="min-h-10 rounded-xl border border-ledger-line bg-ledger-paper px-3 text-sm text-ledger-ink outline-none transition focus:border-ledger-brass" placeholder="org_..." type="text" />
        </label>
      </div>

      <label class="grid gap-2">
        <span class="text-xs text-ledger-muted">Project ID</span>
        <input v-model="form.projectId" class="min-h-10 rounded-xl border border-ledger-line bg-ledger-paper px-3 text-sm text-ledger-ink outline-none transition focus:border-ledger-brass" placeholder="proj_..." type="text" />
      </label>

      <div class="flex flex-col gap-3 pt-1 sm:flex-row">
        <button class="inline-flex min-h-10 items-center justify-center gap-2 rounded-xl bg-ledger-graphite px-3 text-xs font-semibold text-ledger-paper transition hover:bg-ledger-graphite-soft disabled:cursor-not-allowed disabled:opacity-60" :disabled="isLoading" type="submit">
          <KeyRound :size="16" aria-hidden="true" /> Save and validate
        </button>
        <button class="inline-flex min-h-10 items-center justify-center gap-2 rounded-xl border border-ledger-line bg-ledger-paper px-3 text-xs font-semibold text-ledger-ink transition hover:bg-ledger-inset disabled:cursor-not-allowed disabled:opacity-60" :disabled="isLoading || !connection?.hasCredentials" type="button" @click="syncOpenAi">
          <RefreshCw :class="['size-4', isLoading && 'animate-spin']" aria-hidden="true" /> Sync OpenAI
        </button>
      </div>
    </form>

    <dl v-if="connection" class="mt-4 grid gap-3 rounded-2xl bg-ledger-inset p-3 text-xs text-ledger-muted md:grid-cols-2">
      <div>
        <dt>Credentials</dt>
        <dd class="mt-1 font-medium text-ledger-ink">{{ connection.hasCredentials ? "Saved" : "Pending" }}</dd>
      </div>
      <div>
        <dt>Usage access</dt>
        <dd class="mt-1 font-medium text-ledger-ink">{{ connection.usageAccess ? "Available" : "Unconfirmed" }}</dd>
      </div>
      <div>
        <dt>Last validation</dt>
        <dd class="mt-1 font-medium text-ledger-ink">{{ connection.lastValidatedAt ?? "Never" }}</dd>
      </div>
      <div>
        <dt>Last sync</dt>
        <dd class="mt-1 font-medium text-ledger-ink">{{ connection.lastSyncAt ?? "Never" }}</dd>
      </div>
    </dl>

    <p v-if="message" class="mt-4 rounded-2xl border border-emerald-300 bg-emerald-50 px-4 py-3 text-sm text-emerald-800">{{ message }}</p>
    <p v-if="errorMessage" class="mt-4 rounded-2xl border border-red-300 bg-red-50 px-4 py-3 text-sm text-red-800">{{ errorMessage }}</p>
    <p v-if="connection?.lastError" class="mt-4 rounded-2xl border border-amber-300 bg-amber-50 px-4 py-3 text-sm text-amber-800">{{ connection.lastError }}</p>
  </section>
</template>
