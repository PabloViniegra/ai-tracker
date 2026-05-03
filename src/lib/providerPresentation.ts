import type { Confidence, ProviderId, ProviderStatus, UsageSource } from '../types/usage'

interface ProviderLogo {
  readonly src: string | null
  readonly initials: string
  readonly label: string
}

const providerLogos = {
  openai: { src: '/provider-logos/openai.svg', initials: 'OA', label: 'OpenAI' },
  anthropic: { src: '/provider-logos/anthropic.svg', initials: 'A', label: 'Anthropic' },
} satisfies Record<ProviderId, ProviderLogo>

export function getProviderLogo(providerId: ProviderId): ProviderLogo {
  return providerLogos[providerId]
}

export function clampUsagePercent(value: number | null | undefined): number {
  if (value == null || Number.isNaN(value)) {
    return 8
  }

  return Math.min(100, Math.max(8, Math.round(value)))
}

export function sourceLabel(source: UsageSource): string {
  const labels = {
    official_api: 'Official API',
    experimental_local_oauth: 'Experimental',
    manual: 'Manual',
  } satisfies Record<UsageSource, string>

  return labels[source]
}

export function confidenceLabel(confidence: Confidence): string {
  const labels = {
    high: 'High',
    medium: 'Medium',
    low: 'Low',
  } satisfies Record<Confidence, string>

  return labels[confidence]
}

export function statusLabel(status: ProviderStatus): string {
  const labels = {
    connected: 'Connected',
    needs_credentials: 'Needs credentials',
  } satisfies Record<ProviderStatus, string>

  return labels[status]
}
