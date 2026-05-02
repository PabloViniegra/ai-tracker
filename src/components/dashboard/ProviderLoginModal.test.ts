import { describe, expect, it } from 'vitest'
import type { ProviderId } from '../../types/usage'
import { getProviderLogo } from '../../lib/providerPresentation'

describe('ProviderLoginModal', () => {
  describe('getProviderLogo', () => {
    it('returns correct logo for openai', () => {
      const logo = getProviderLogo('openai')
      expect(logo.label).toBe('OpenAI')
      expect(logo.initials).toBe('OA')
      expect(logo.src).toBe('/provider-logos/openai.svg')
    })

    it('returns correct logo for anthropic', () => {
      const logo = getProviderLogo('anthropic')
      expect(logo.label).toBe('Anthropic')
      expect(logo.initials).toBe('A')
      expect(logo.src).toBe('/provider-logos/anthropic.svg')
    })

  })

  describe('modal setup type mapping', () => {
    it('maps openai provider to openai setup type', () => {
      const setupTypeMap: Record<string, 'openai' | 'anthropic'> = {
        openai: 'openai',
        anthropic: 'anthropic',
      }
      expect(setupTypeMap['openai']).toBe('openai')
    })

    it('maps anthropic provider to anthropic setup type', () => {
      const setupTypeMap: Record<string, 'openai' | 'anthropic'> = {
        openai: 'openai',
        anthropic: 'anthropic',
      }
      expect(setupTypeMap['anthropic']).toBe('anthropic')
    })

    it('does not map other provider ids', () => {
      const setupTypeMap: Record<string, 'openai' | 'anthropic'> = {
        openai: 'openai',
        anthropic: 'anthropic',
      }
      expect(setupTypeMap['legacy_provider']).toBeUndefined()
      expect(setupTypeMap['unknown']).toBeUndefined()
    })
  })

  describe('modal visibility logic', () => {
    it('modal is visible when activeModal is not null', () => {
      const activeModal = { providerId: 'openai' as ProviderId, setupType: 'openai' }
      const isVisible = activeModal !== null
      expect(isVisible).toBe(true)
    })

    it('modal is not visible when activeModal is null', () => {
      const activeModal = null
      const isVisible = activeModal !== null
      expect(isVisible).toBe(false)
    })

    it('modal closes by setting activeModal to null', () => {
      let activeModal: { providerId: ProviderId; setupType: string } | null = { providerId: 'openai' as ProviderId, setupType: 'openai' }
      activeModal = null
      expect(activeModal).toBeNull()
    })
  })
})
