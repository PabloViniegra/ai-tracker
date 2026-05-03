<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { getDashboardModalStyle } from '../../lib/dashboardWindow'
import type { ProviderId } from '../../types/usage'

type ProviderSetupType = 'openai' | 'anthropic'

defineProps<{
  providerId: ProviderId
  setupType: ProviderSetupType
  visible: boolean
}>()

const emit = defineEmits<{
  close: []
}>()

const _dialogStyle = getDashboardModalStyle()

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    emit('close')
  }
}

function _handleBackdropClick(event: MouseEvent) {
  if (event.target === event.currentTarget) {
    emit('close')
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition-opacity duration-200 ease-out"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition-opacity duration-150 ease-in"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="visible"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4 backdrop-blur-sm"
        role="dialog"
        aria-modal="true"
        :aria-labelledby="`modal-title-${providerId}`"
        @click="handleBackdropClick"
      >
        <div class="w-full max-w-md overflow-y-auto rounded-2xl border border-ledger-line bg-ledger-panel p-6 shadow-2xl" :style="dialogStyle">
          <div class="mb-4 flex items-center justify-between">
            <div class="flex items-center gap-3">
              <span class="grid size-10 shrink-0 place-items-center rounded-xl border border-ledger-line bg-ledger-paper">
                <img v-if="getProviderLogo(providerId).src" class="size-6" :src="getProviderLogo(providerId).src ?? ''" :alt="`${getProviderLogo(providerId).label} logo`" />
                <span v-else class="text-xs font-bold text-ledger-ink" aria-hidden="true">{{ getProviderLogo(providerId).initials }}</span>
              </span>
              <h2 :id="`modal-title-${providerId}`" class="text-lg font-semibold text-ledger-ink">
                Connect {{ getProviderLogo(providerId).label }}
              </h2>
            </div>
            <button
              class="rounded-lg p-1.5 text-ledger-muted transition-colors hover:bg-ledger-inset hover:text-ledger-ink"
              type="button"
              aria-label="Close modal"
              @click="emit('close')"
            >
              <X :size="20" aria-hidden="true" />
            </button>
          </div>

          <slot />
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
