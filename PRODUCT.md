# AI Tracker Product Brief

AI Tracker is a Windows desktop app for tracking AI subscription token usage across multiple providers from a local-first Tauri application.

## MVP Scope

- Track daily and weekly AI usage from supported providers.
- Keep credentials private on the user's machine.
- Prefer official provider APIs when reliable usage endpoints exist.
- Prepare for local estimates or experimental connectors when providers do not expose public usage APIs.
- Show whether each metric is official, estimated, or manual.

## Target Providers

- OpenAI
- Anthropic
- Google Gemini
- GitHub Copilot
- Opencode
- Kimi
- Minimax
- GLM
- Cursor

## Product Principles

- Privacy first: no backend cloud in the MVP.
- Accuracy transparency: every usage value has a source and confidence level.
- Connector modularity: providers can differ in capabilities without breaking the dashboard.
- Local history: usage snapshots are retained locally for daily and weekly trends.
- Real-time enough: use polling and manual refresh, not constant streaming.

## MVP Success Criteria

- The app launches as a Tauri Windows desktop app.
- The dashboard shows provider status, token totals, cost, confidence, and sync recency.
- Provider data flows through Tauri commands instead of hard-coded UI-only state.
- The architecture can add real OpenAI, Anthropic, and Gemini connectors next.
- The UI communicates limitations for experimental providers clearly.
