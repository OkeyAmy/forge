# Forge landing — TestSprite PRD (frontend)

## Overview

The **landing** app is a **Next.js** marketing site: one route **`/`** with sections addressable via hashes **`#features`**, **`#how-it-works`**, **`#setup`**, **`#model-config`**.

## Automated test policy (required for TestSprite)

**Clicks**

- The only **mandatory external click** targets are **GitHub** links (hero, navbar, footer). Prefer verifying **`href`**, **`target="_blank"`**, and **`rel`** (must include **`noopener`** and **`noreferrer`**) on the **landing page** before relying on opening github.com.
- **Do not** require automated clicks on the **Docker / Build from source** switcher or on **model provider** tab buttons. For those areas, tests should only check that content and controls **are visible** after scroll or hash navigation.

**Navigation**

- Allowed: **scroll**, **`/#section`** URLs, and normal **in-page** anchor links (e.g. **Get Started** → `#setup`).

**External pages**

- If the runner cannot load **github.com** or partner sites, **do not fail** solely for that reason when markup on the landing origin is correct.

## Out of scope

- **forge-api**, **`/api/*`**, running Forge or Docker from the browser.
