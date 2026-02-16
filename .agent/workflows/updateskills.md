---
description: Pull the latest Antigravity Awesome Skills from the repo
---

# Update Antigravity Skills

Pulls the latest skills from `sickn33/antigravity-awesome-skills` into the global Antigravity skills directory.

## Architecture

- **Repo clone**: `C:\Users\admin-beats\.gemini\antigravity\antigravity-awesome-skills-repo`
- **Skills junction**: `C:\Users\admin-beats\.gemini\antigravity\skills` → repo's `skills\` subfolder
- Git pull updates the repo; the junction makes skills immediately available at the correct path.

## Steps

// turbo

1. Pull latest changes from the repo:

```
git -C "C:\Users\admin-beats\.gemini\antigravity\antigravity-awesome-skills-repo" pull
```

// turbo 2. Verify the skills directory is still linked correctly:

```
Test-Path "C:\Users\admin-beats\.gemini\antigravity\skills\antigravity-workflows\SKILL.md"
```

// turbo 3. Count installed skills to confirm:

```
(Get-ChildItem "C:\Users\admin-beats\.gemini\antigravity\skills" -Directory).Count
```

4. Report the result: how many skills are installed and whether any new skills were added.
