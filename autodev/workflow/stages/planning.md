# Implementation planning

Read the selected issue, repository instructions, specification, technical plan, tasks and project constitution when present. Identify unmet prerequisites and stop with blockers instead of selecting a different issue. Preserve the issue selected by Rust.

Break implementation into dependency-ordered subtasks. Assign each to the appropriate role under workflow/roles: backend, frontend, database, async processing, integrations, infrastructure, documentation, quality. Identify shared files, API contracts and migration risks before delegating. Record the plan in the feature's plan/tasks artifacts. Do not implement during this stage.

Define checks for each acceptance criterion and any security, tenant-isolation or migration requirement. Do not invent project-specific requirements. The next stage delegates and integrates the work; the Rust engine owns all Git publication. Report missing delegation capability as a blocker when delegation is required.
