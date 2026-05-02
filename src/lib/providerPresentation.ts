import type { Confidence, ProviderId, ProviderStatus, UsageSource } from '../types/usage'

interface ProviderLogo {
  readonly src: string | null
  readonly initials: string
  readonly label: string
}

const providerLogos = {
  openai: { src: null, initials: 'OA', label: 'OpenAI' },
  anthropic: { src: '/provider-logos/anthropic.svg', initials: 'A', label: 'Anthropic' },
  gemini: { src: '/provider-logos/google-gemini.svg', initials: 'G', label: 'Google Gemini' },
  github_copilot: {
    src: '/provider-logos/github-copilot.svg',
    initials: 'GH',
    label: 'GitHub Copilot',
  },
  opencode: { src: null, initials: 'OC', label: 'Opencode' },
  kimi: { src: null, initials: 'K', label: 'Kimi' },
  minimax: { src: '/provider-logos/minimax.svg', initials: 'MM', label: 'MiniMax' },
  glm: { src: null, initials: 'GL', label: 'GLM' },
  cursor: { src: '/provider-logos/cursor.svg', initials: 'C', label: 'Cursor' },
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
    local_estimate: 'Local estimate',
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
    experimental: 'Experimental',
    unsupported: 'Unsupported',
  } satisfies Record<ProviderStatus, string>

  return labels[status]
}
