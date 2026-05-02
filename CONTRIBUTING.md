# Contributing to AI Tracker

Thank you for your interest in contributing. All contributions are accepted through the fork workflow to keep the main branch stable.

## Fork Workflow

1. **Fork the repository** on GitHub.

2. **Clone your fork** locally:

   ```bash
   git clone https://github.com/your-username/ai-tracker.git
   cd ai-tracker
   ```

3. **Add the upstream remote** to keep your fork in sync:

   ```bash
   git remote add upstream https://github.com/original-owner/ai-tracker.git
   ```

4. **Create a feature branch** from the latest `main`:

   ```bash
   git checkout main
   git pull upstream main
   git checkout -b feature/your-feature-name
   ```

5. **Make your changes**. Follow the conventions described below.

6. **Run tests and type checks** before committing:

   ```bash
   pnpm test
   pnpm build
   ```

7. **Commit with a clear message**. Use imperative mood and keep the subject line under 72 characters:

   ```
   Add OpenAI usage connector
   ```

8. **Push to your fork** and open a pull request:

   ```bash
   git push origin feature/your-feature-name
   ```

## Contribution Guidelines

### Code Conventions

- Vue components must stay under 200 lines. Use Composition API with `<script setup>` and TypeScript.
- Tailwind CSS v4 is the styling layer. Avoid plain CSS unless strictly necessary.
- Rust backend code follows standard Rust conventions (edition 2021).
- Keep components focused: one responsibility per file.

### Pull Requests

- Describe **what** changed and **why**.
- Link any related issues.
- Include screenshots or recordings for UI changes.
- Ensure all tests pass and type checks succeed.
- One logical change per PR. Do not mix unrelated refactors.

### Commit Messages

- Use imperative mood: "Add", "Fix", "Update", "Remove".
- Prefix with scope when applicable: `docs:`, `ui:`, `backend:`, `test:`.
- Keep the first line under 72 characters. Add a body for context if needed.

### Reporting Issues

- Search existing issues before opening a new one.
- Include steps to reproduce, expected behavior, and actual behavior.
- Attach screenshots for visual bugs.
- Specify your OS and application version.

### Code of Conduct

- Be respectful in all discussions.
- Review feedback constructively.
- If asked to make changes, address them in new commits rather than force-pushing over reviewed history unless requested.

## Getting Help

If you are unsure about anything, open an issue to discuss your idea before writing code. It is easier to align on direction early than to rework a completed PR.
