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
    errorMessage.value = error instanceof Error ? error.message : "No se pudo leer la configuracion de OpenAI";
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
    errorMessage.value = error instanceof Error ? error.message : "No se pudo guardar la API key";
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
    message.value = "Sincronizacion solicitada. Revisa el timeline para ver si OpenAI devolvio uso real o una advertencia de permisos.";
    emit("updated");
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : "No se pudo sincronizar OpenAI";
  } finally {
    isLoading.value = false;
  }
}

onMounted(() => {
  void loadConnection();
});
</script>

<template>
  <section class="rounded-3xl border border-slate-800 bg-slate-900/60 p-5" id="settings" aria-labelledby="openai-title">
    <div class="flex items-start justify-between gap-3">
      <div>
        <p class="font-mono text-xs uppercase tracking-[0.22em] text-slate-500">Vault + SQLite</p>
        <h2 id="openai-title" class="mt-1 text-xl font-semibold text-slate-50">Conectar OpenAI</h2>
      </div>
      <span class="inline-flex items-center gap-2 rounded-full border border-emerald-400/20 bg-emerald-400/10 px-3 py-1 text-xs text-emerald-200">
        <Shield :size="14" aria-hidden="true" /> keyring local
      </span>
    </div>

    <p class="mt-4 text-sm leading-6 text-slate-400">
      La API key se guarda en el keyring del sistema. La metadata y el historico quedan en SQLite local.
      Para consumo real, OpenAI suele exigir una <span class="font-semibold text-slate-200">Admin Key</span> en los endpoints de usage/costs.
    </p>

    <form class="mt-5 grid gap-3" @submit.prevent="saveCredentials">
      <label class="grid gap-2">
        <span class="text-sm text-slate-300">API key</span>
        <input v-model="form.apiKey" class="min-h-11 rounded-xl border border-slate-700 bg-slate-950 px-3 text-slate-100 outline-none transition focus:border-amber-400" placeholder="sk-..." type="password" autocomplete="off" />
      </label>

      <div class="grid gap-3 md:grid-cols-2">
        <label class="grid gap-2">
          <span class="text-sm text-slate-300">Account label</span>
          <input v-model="form.accountLabel" class="min-h-11 rounded-xl border border-slate-700 bg-slate-950 px-3 text-slate-100 outline-none transition focus:border-amber-400" placeholder="Personal / Team" type="text" />
        </label>
        <label class="grid gap-2">
          <span class="text-sm text-slate-300">Organization ID</span>
          <input v-model="form.organizationId" class="min-h-11 rounded-xl border border-slate-700 bg-slate-950 px-3 text-slate-100 outline-none transition focus:border-amber-400" placeholder="org_..." type="text" />
        </label>
      </div>

      <label class="grid gap-2">
        <span class="text-sm text-slate-300">Project ID</span>
        <input v-model="form.projectId" class="min-h-11 rounded-xl border border-slate-700 bg-slate-950 px-3 text-slate-100 outline-none transition focus:border-amber-400" placeholder="proj_..." type="text" />
      </label>

      <div class="flex flex-col gap-3 pt-1 sm:flex-row">
        <button class="inline-flex min-h-11 items-center justify-center gap-2 rounded-xl bg-amber-500 px-4 text-sm font-semibold text-slate-950 transition hover:bg-amber-400 disabled:cursor-not-allowed disabled:opacity-60" :disabled="isLoading" type="submit">
          <KeyRound :size="16" aria-hidden="true" /> Guardar y validar
        </button>
        <button class="inline-flex min-h-11 items-center justify-center gap-2 rounded-xl border border-slate-700 bg-slate-950 px-4 text-sm font-semibold text-slate-100 transition hover:border-blue-400/40 hover:text-blue-100 disabled:cursor-not-allowed disabled:opacity-60" :disabled="isLoading || !connection?.hasCredentials" type="button" @click="syncOpenAi">
          <RefreshCw :class="['size-4', isLoading && 'animate-spin']" aria-hidden="true" /> Sincronizar OpenAI
        </button>
      </div>
    </form>

    <dl v-if="connection" class="mt-5 grid gap-3 rounded-2xl border border-slate-800 bg-slate-950/60 p-4 text-sm text-slate-300 md:grid-cols-2">
      <div>
        <dt class="text-slate-500">Credenciales</dt>
        <dd class="mt-1 font-medium text-slate-100">{{ connection.hasCredentials ? "Guardadas" : "Pendientes" }}</dd>
      </div>
      <div>
        <dt class="text-slate-500">Usage access</dt>
        <dd class="mt-1 font-medium text-slate-100">{{ connection.usageAccess ? "Disponible" : "Sin confirmar" }}</dd>
      </div>
      <div>
        <dt class="text-slate-500">Ultima validacion</dt>
        <dd class="mt-1 font-medium text-slate-100">{{ connection.lastValidatedAt ?? "Nunca" }}</dd>
      </div>
      <div>
        <dt class="text-slate-500">Ultimo sync</dt>
        <dd class="mt-1 font-medium text-slate-100">{{ connection.lastSyncAt ?? "Nunca" }}</dd>
      </div>
    </dl>

    <p v-if="message" class="mt-4 rounded-2xl border border-emerald-400/30 bg-emerald-400/10 px-4 py-3 text-sm text-emerald-100">{{ message }}</p>
    <p v-if="errorMessage" class="mt-4 rounded-2xl border border-red-400/30 bg-red-400/10 px-4 py-3 text-sm text-red-100">{{ errorMessage }}</p>
    <p v-if="connection?.lastError" class="mt-4 rounded-2xl border border-amber-400/30 bg-amber-400/10 px-4 py-3 text-sm text-amber-100">{{ connection.lastError }}</p>
  </section>
</template>
