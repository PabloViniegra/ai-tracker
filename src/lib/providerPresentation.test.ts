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
    expect(getProviderLogo('openai')).toMatchObject({ src: '/provider-logos/openai.svg', initials: 'OA', label: 'OpenAI' })
  })

  it('returns correct logo for the active providers', () => {
    expect(getProviderLogo('openai').src).toBe('/provider-logos/openai.svg')
    expect(getProviderLogo('anthropic').src).toBe('/provider-logos/anthropic.svg')
  })

  it('returns correct initials for the active providers', () => {
    expect(getProviderLogo('openai').initials).toBe('OA')
    expect(getProviderLogo('anthropic').initials).toBe('A')
  })

  it('keeps source, confidence, and status labels explicit', () => {
    expect(sourceLabel('official_api')).toBe('Official API')
    expect(confidenceLabel('high')).toBe('High')
    expect(statusLabel('needs_credentials')).toBe('Needs credentials')
  })
})
