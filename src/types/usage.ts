export type ProviderId =
  | 'openai'
  | 'anthropic'

export type UsageSource = 'official_api'
export type Confidence = 'high'
export type ProviderStatus = 'connected' | 'needs_credentials'

export interface ProviderCapabilities {
  tokens: boolean
  cost: boolean
  quota: boolean
  realtime: boolean
  historical: boolean
}

export interface ProviderSummary {
  id: ProviderId
  name: string
  status: ProviderStatus
  source: UsageSource
  confidence: Confidence
  capabilities: ProviderCapabilities
  dailyTokens: number
  weeklyTokens: number
  costUsd: number | null
  quotaUsed: number | null
  quotaLimit: number | null
  lastSync: string | null
}

export interface UsagePoint {
  day: string
  tokens: number
  costUsd: number
}

export interface SyncEvent {
  providerId: ProviderId
  providerName: string
  status: 'success' | 'warning' | 'error'
  message: string
  at: string
}

export interface DashboardSnapshot {
  providers: ProviderSummary[]
  history: UsagePoint[]
  syncEvents: SyncEvent[]
}

export interface OpenAiConnectionState {
  hasCredentials: boolean
  accountLabel: string | null
  organizationId: string | null
  projectId: string | null
  lastValidatedAt: string | null
  lastSyncAt: string | null
  lastError: string | null
  usageAccess: boolean
}

export interface SaveOpenAiCredentialsInput {
  apiKey: string
  accountLabel: string | null
  organizationId: string | null
  projectId: string | null
}

export interface SaveOpenAiCredentialsResult {
  connection: OpenAiConnectionState
  message: string
}

export interface AnthropicConnectionState {
  hasCredentials: boolean
  accountLabel: string | null
  lastValidatedAt: string | null
  lastSyncAt: string | null
  lastError: string | null
  usageAccess: boolean
}

export interface SaveAnthropicCredentialsInput {
  apiKey: string
  accountLabel: string | null
}

export interface SaveAnthropicCredentialsResult {
  connection: AnthropicConnectionState
  message: string
}
