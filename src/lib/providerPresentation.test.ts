import { describe, expect, it } from 'vitest'
import {
  clampUsagePercent,
  confidenceLabel,
  getProviderLogo,
  sourceLabel,
  statusLabel,
} from './providerPresentation'

describe('providerPresentation', () => {
  it('clamps usage strip values into a visible safe range', () => {
    expect(clampUsagePercent(null)).toBe(8)
    expect(clampUsagePercent(Number.NaN)).toBe(8)
    expect(clampUsagePercent(2)).toBe(8)
    expect(clampUsagePercent(42.4)).toBe(42)
    expect(clampUsagePercent(140)).toBe(100)
  })

  it('returns provider logo metadata and fallback identity', () => {
    expect(getProviderLogo('anthropic')).toMatchObject({
      src: '/provider-logos/anthropic.svg',
      label: 'Anthropic',
    })
    expect(getProviderLogo('openai')).toMatchObject({ src: null, initials: 'OA', label: 'OpenAI' })
  })

  it('keeps source, confidence, and status labels explicit', () => {
    expect(sourceLabel('official_api')).toBe('Official API')
    expect(sourceLabel('local_estimate')).toBe('Local estimate')
    expect(confidenceLabel('low')).toBe('Low')
    expect(statusLabel('needs_credentials')).toBe('Needs credentials')
  })
})
