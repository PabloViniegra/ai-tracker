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

  it('returns correct logo for all providers with official icons', () => {
    expect(getProviderLogo('openai').src).toBe('/provider-logos/openai.svg')
    expect(getProviderLogo('anthropic').src).toBe('/provider-logos/anthropic.svg')
    expect(getProviderLogo('gemini').src).toBe('/provider-logos/google-gemini.svg')
    expect(getProviderLogo('github_copilot').src).toBe('/provider-logos/github-copilot.svg')
    expect(getProviderLogo('minimax').src).toBe('/provider-logos/minimax.svg')
    expect(getProviderLogo('cursor').src).toBe('/provider-logos/cursor.svg')
  })

  it('returns null src for providers without official icons', () => {
    expect(getProviderLogo('opencode').src).toBeNull()
    expect(getProviderLogo('kimi').src).toBeNull()
    expect(getProviderLogo('glm').src).toBeNull()
  })

  it('returns correct initials for all providers', () => {
    expect(getProviderLogo('openai').initials).toBe('OA')
    expect(getProviderLogo('anthropic').initials).toBe('A')
    expect(getProviderLogo('gemini').initials).toBe('G')
    expect(getProviderLogo('github_copilot').initials).toBe('GH')
    expect(getProviderLogo('opencode').initials).toBe('OC')
    expect(getProviderLogo('kimi').initials).toBe('K')
    expect(getProviderLogo('minimax').initials).toBe('MM')
    expect(getProviderLogo('glm').initials).toBe('GL')
    expect(getProviderLogo('cursor').initials).toBe('C')
  })

  it('keeps source, confidence, and status labels explicit', () => {
    expect(sourceLabel('official_api')).toBe('Official API')
    expect(sourceLabel('local_estimate')).toBe('Local estimate')
    expect(confidenceLabel('low')).toBe('Low')
    expect(statusLabel('needs_credentials')).toBe('Needs credentials')
  })
})
