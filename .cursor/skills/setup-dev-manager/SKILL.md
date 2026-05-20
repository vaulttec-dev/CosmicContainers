---
name: setup-dev-manager
description: >-
  Configure SnakeFlow (Dev Environment Manager) for a project by scanning the repo
  and writing devManager.* to .vscode/settings.json. Use for a new PC, fresh clone,
  onboarding, or when the user wants automatic setup of servers, Docker, database,
  Quality Hub, or mentions dev-environment-manager / devManager settings.
---

# Setup Dev Environment Manager

## Step 0 — Fetch the Agent Guide (REQUIRED)

**Before doing anything else**, fetch the full agent instruction set from the live documentation:

```
https://snakeflow.pages.dev/getting-started/cursor-setup-skill/
```

That page is the **single source of truth** for this skill. It contains:
- What files to scan (Phase 1 — Project Exploration)
- All 50 questions to ask the user (Phase 2), grouped by topic
- How to write `.vscode/settings.json` (Phase 3) with all settings examples and rules
- Full examples for Next.js, Turborepo, Django+React, Go microservices, and more

After fetching, follow the workflow described there **exactly**.

---

## Quick Reference (offline fallback)

Use this section **only if the docs URL is unreachable**.

### Existing settings — review and update mode

**If `.vscode/settings.json` already contains `devManager.*` keys:**

1. Read the entire existing file first.
2. For every existing key, verify it is still valid against the current docs — settings names and defaults change as SnakeFlow evolves.
3. Identify and fix:
   - **Renamed keys** — old name no longer works; replace with current name.
   - **Removed settings** — delete orphaned keys that no longer exist.
   - **New required settings** — keys that must now be present but are missing.
   - **Outdated values** — defaults or allowed values that changed.
4. Ask the user about any settings that cannot be inferred automatically (same Q&A flow as a fresh setup, but skip questions already answered by existing values).
5. Write the merged, corrected settings back. **Never silently drop existing intentional values** — always confirm before removing a key that was set by the user.

This review mode applies every time the skill runs, not only on first setup.

---

### What this skill does

1. **Explores the project** — lockfiles, `package.json`, Docker Compose, ORM, scripts, monorepo layout, component patterns, layer structure, existing settings
2. **Summarizes findings** to the user (5–10 lines) including reviewer integration status
3. **Asks 50 questions** grouped into 10 topic groups (A–J) — skip only questions that are already definitively answered by existing settings
4. **Writes or updates `.vscode/settings.json`** — all `devManager.*` keys in one place

> `.vscode/settings.json` is the **only** place for all `devManager.*` settings. Never create separate config files.

---

### Key detection targets

```
Package manager:  pnpm-lock.yaml → pnpm | bun.lockb → bun | yarn.lock → yarn | package-lock.json → npm
Monorepo:         turbo.json, nx.json, pnpm-workspace.yaml, package.json "workspaces"
Docker Compose:   docker-compose.yaml / compose.yaml in root, docker/, infra/
ORM / DB schema:  prisma/schema.prisma → Prisma | drizzle-orm → Drizzle | alembic.ini / models.py → Django/SQLAlchemy
                  db/schema.rb → Rails | database/migrations/*.php → Laravel | *.csproj → EF Core
                  go.mod (gorm.io/gorm or entgo.io/ent) → Go | migrations/*.sql → Raw SQL
DB CLI tools:     prisma-lint in devDeps | squawk/squawk-cli in deps | atlas in PATH
Scripts:          package.json "scripts", Makefile, Cargo.toml, pyproject.toml
Architecture:     components/, hooks/, services/, controllers/, api/, stores/, dto/, entities/
Duplicates:       scan *.tsx/.jsx/.vue/.svelte for Button/Modal/Form/Table/Card exports (≥2 = warn)
Reviewer config:  see GitHub Reviewer Integration Registry below — scan for ALL config files
Existing settings: .vscode/settings.json — read first, review all devManager.* keys (see above)
```

---

### GitHub Reviewer Integration Registry

**Always scan for these files during Phase 1. Never skip.** Report PRESENT / MISSING for every service. Include this table in the Phase 1 summary shown to the user.

| Service | SnakeFlow provider ID | Config file(s) to check | Notes |
|---|---|---|---|
| CodeRabbit | `coderabbit` | `.coderabbit.yaml` | PR review bot — requires `apiKey` |
| SonarCloud | `sonarcloud` | `sonar-project.properties` | requires `token` + `projectKey` |
| Snyk | `snyk` | `.snyk` | requires `token` + `orgId` |
| Codecov | `codecov` | `codecov.yml` or `.codecov.yml` | requires `token` + `owner` + `repo` |
| Coveralls | `coveralls` | `.coveralls.yml` | requires `owner` + `repo` |
| Code Climate | `codeclimate` | `.codeclimate.yml` | requires `token` + `repoSlug` |
| Codacy | `codacy` | `.codacy.yml` | requires `token` + `org` + `repo` |
| DeepSource | `deepsource` | `.deepsource.toml` | requires `token` + `login` + `repo` |
| Semgrep | `semgrep` (builtin CLI) | `.semgrep.yml` or `.semgrep/` dir | local SAST — no token |
| CodeQL | `codeql` | `.github/workflows/codeql.yml` or `.github/codeql/` dir | uses VS Code GitHub auth |
| Qlty | `qlty` | `.qlty/qlty.toml` | requires `token` + `owner` + `project` |
| CodeAnt AI | `codeant` | *(no local file)* | requires `token`; `repoId` auto from git |
| Aikido | `aikido` | *(no local file — GitHub App)* | requires `token` |
| Greptile | `greptile` | *(no local file — GitHub App)* | requires `apiKey` + `githubToken` |
| Qodo | `qodo` | *(no local file — GitHub App)* | uses VS Code GitHub auth only |
| Cubic | `cubic` | *(no local file)* | requires `apiKey` |
| Atlas | `atlas` (builtin CLI) | `atlas.hcl` | migration safety — no token |

**For each service:**
- If config file is PRESENT → offer to enable `devManager.quality.{id}.enabled: true` and configure required token/fields.
- If config file is MISSING → note it as not set up; offer setup link if user wants to add it.
- Cloud-only services (no config file) → check if `devManager.quality.{id}.enabled` is already set in existing settings.

**Quick-start links:**
- CodeRabbit → https://docs.coderabbit.ai
- SonarCloud → https://sonarcloud.io/documentation/getting-started/github/
- Snyk → https://docs.snyk.io/scm-ide-and-ci-cd-integrations/snyk-scm-integrations/github
- Codecov → https://docs.codecov.com/docs/quick-start
- Code Climate → https://codeclimate.com/github/
- DeepSource → https://deepsource.com/docs/integrations/github/
- CodeQL → https://docs.github.com/en/code-security/code-scanning
- Qlty → https://docs.qlty.sh/getting-started

---

### 10 question groups

| Group | Topics | Questions |
|-------|--------|-----------|
| A | Project identity, exclusions | Q1–Q6 |
| B | Dev servers, workers, Git | Q7–Q12 |
| C | Docker, infrastructure | Q13–Q17 |
| D | Database, ORM | Q18–Q21 |
| E | Custom checks, scripts | Q22–Q26 |
| F | Architecture: layers, component inventory, fileRules, folderRules, coupling, complexity | Q27–Q34 |
| G | Built-in checks to disable | Q35–Q40 |
| H | Security & CLI tools, DB schema lint | Q41–Q45 (incl. Q44b) |
| I | Cloud providers (use Registry above for each) | Q46–Q48 |
| J | Git, CI/CD, Gemini AI | Q49–Q50 |

**Do not skip any group.** If existing settings already cover a question, confirm the value is still correct instead of re-asking.

---

### Critical architecture questions (Group F)

**Q28 — Component patterns** (duplicate component detection):
Built-in: Button, Modal, Dialog, Table, Card, Input, Form, Select, Dropdown, List, Tab, Badge, Avatar, Tooltip, Loader, Notification, Header/Navbar, Sidebar, Chart, Icon, Pagination, Stepper, useAuth, useFetch, useForm, useModal, useFilter, useSort, usePagination, useDebounce, useStorage.
Ask for project-specific additions.

**Q31 — fileRules** (files must be in specific folders):
```jsonc
"devManager.quality.builtin.projectStructure.fileRules": [
  { "filePattern": "*.service.ts",    "mustBeIn": "src/services",     "description": "Services → src/services/" },
  { "filePattern": "*.controller.ts", "mustBeIn": "src/controllers",  "description": "Controllers → src/controllers/" },
  { "filePattern": "*.repository.ts", "mustBeIn": "src/repositories", "description": "Repos → src/repositories/" },
  { "filePattern": "*.store.ts",      "mustBeIn": "src/stores",       "description": "Stores → src/stores/" },
  { "filePattern": "*.dto.ts",        "mustBeIn": "src/dto",          "description": "DTOs → src/dto/" },
  { "filePattern": "*.entity.ts",     "mustBeIn": "src/entities",     "description": "Entities → src/entities/" },
  { "filePattern": "*_handler.go",    "mustBeIn": "internal/handlers","description": "Handlers → internal/handlers/" }
]
```

**Q32 — folderRules** (files inside a folder must export specific symbols):
```jsonc
"devManager.quality.builtin.projectStructure.folderRules": [
  { "folderPattern": "src/hooks",      "mustExport": "export function use | export const use", "description": "Hooks must export useXxx" },
  { "folderPattern": "src/services",   "mustExport": "export class | export function | export const" },
  { "folderPattern": "src/components", "mustExport": "export default | export const" }
]
```

**Q29 — componentInventory minCountOverrides + customPatterns**:
```jsonc
"devManager.quality.builtin.componentInventory.minCountOverrides": { "Button": 3 },
"devManager.quality.builtin.componentInventory.customPatterns": [
  { "pattern": "Panel", "label": "Panel", "suggestion": "unified Panel component", "kind": "component", "minCount": 2 }
]
```

---

### Settings writing rules

- **Always write**: `project.name`, `project.dir`, `project.packageManager`, `project.servers`
- **Skip defaults**: omit settings equal to documented defaults
- **Skip empty**: no empty arrays, null values, or empty strings for optional fields
- **Skip cloud blocks** without tokens — but note them as available
- **Path**: `.vscode/settings.json` — the ONLY place for `devManager.*`
- **Existing values**: preserve user-set values unless confirmed outdated or renamed

---

### Full documentation links

| Topic | URL |
|-------|-----|
| Dev Servers | https://snakeflow.pages.dev/features/servers/ |
| Docker Compose | https://snakeflow.pages.dev/features/containers/ |
| Database / ORM | https://snakeflow.pages.dev/features/database/ |
| Quality Hub | https://snakeflow.pages.dev/quality/ |
| Built-in Checks (70+) | https://snakeflow.pages.dev/quality/builtin-checks/ |
| CLI Tool Checks | https://snakeflow.pages.dev/quality/cli-tools/ |
| Cloud Providers (15) | https://snakeflow.pages.dev/quality/cloud-providers/ |
| Custom Checks | https://snakeflow.pages.dev/quality/custom-checks/ |
| All Settings Reference | https://snakeflow.pages.dev/reference/configuration/ |
| CLI Tools Install | https://snakeflow.pages.dev/reference/cli-install/ |
