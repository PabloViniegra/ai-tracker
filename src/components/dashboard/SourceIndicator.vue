<script setup lang="ts">
import type { Confidence, UsageSource } from '../../types/usage'

defineProps<{
  source: UsageSource
  confidence: Confidence
  lastSync: string | null
  isWarning?: boolean
}>()

function _formatRelativeTime(isoString: string | null): string {
  if (!isoString) return 'never'
  const date = new Date(isoString)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  if (diffMins < 1) return 'just now'
  if (diffMins < 60) return `${diffMins}m ago`
  const diffHours = Math.floor(diffMins / 60)
  if (diffHours < 24) return `${diffHours}h ago`
  const diffDays = Math.floor(diffHours / 24)
  return `${diffDays}d ago`
}
</script>

<template>
  <div class="flex flex-wrap items-center gap-1.5">
    <span class="rounded-full bg-ledger-inset px-2 py-1 text-[0.68rem] text-ledger-ink">
      {{ sourceLabel(source) }}
    </span>
    <span class="rounded-full bg-ledger-inset px-2 py-1 text-[0.68rem] text-ledger-ink">
      Conf. {{ confidenceLabel(confidence) }}
    </span>
    <span class="flex items-center gap-1 rounded-full bg-ledger-inset px-2 py-1 text-[0.68rem] text-ledger-muted">
      <Clock :size="10" aria-hidden="true" />
      {{ formatRelativeTime(lastSync) }}
    </span>
    <span
      v-if="isWarning || source === 'experimental_local_oauth'"
      class="flex items-center gap-1 rounded-full bg-ledger-brass/20 px-2 py-1 text-[0.68rem] text-ledger-brass"
    >
      <AlertTriangle :size="10" aria-hidden="true" />
      Experimental
    </span>
  </div>
</template>
