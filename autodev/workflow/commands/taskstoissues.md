> Execution contract: This is a backend-neutral workflow instruction. The Rust engine owns Git commits, push and PR creation. Do not execute publication or branch-changing examples from this document inside an agent stage. Use repository-specific checks and policies; examples are not proof of validation. Delegate using the configured backend when supported; otherwise report the missing capability. Rust command syntax in workflow/README.md takes precedence over historical invocation examples.

---
description: Convert existing tasks into actionable, dependency-ordered GitHub issues for the feature based on available design artifacts.
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Outline

1. Run `autodev project prerequisites FEATURE_DIR --require-tasks` from repo root and parse FEATURE_DIR and AVAILABLE_DOCS list. All paths must be absolute. For single quotes in args like "I'm Groot", use escape syntax: e.g 'I'\''m Groot' (or double-quote if possible: "I'm Groot").
1. From the executed script, extract the path to **tasks**.
1. Get the Git remote by running:

```bash
git config --get remote.origin.url
```

**ONLY PROCEED TO NEXT STEPS IF THE REMOTE IS A GITHUB URL**

1. Review each task for a unique ID, dependency ordering, phase/story/priority metadata and acceptance criteria. Produce or correct FEATURE_DIR/tasks.md. Rust publishes reviewed tasks through `autodev project issues FEATURE_DIR/tasks.md`; do not call a vendor-specific API or create issues from this prompt.
