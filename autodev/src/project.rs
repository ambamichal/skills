use super::*;

#[derive(Subcommand)]
pub enum Action {
    /// Infer technical and task metadata labels for open GitHub issues.
    IssueLabels {
        #[arg(long)]
        dry_run: bool,
    },
    /// Create a numbered feature branch and specification scaffold.
    Feature {
        name: String,
        #[arg(long)]
        number: Option<u64>,
        #[arg(long)]
        dry_run: bool,
    },
    /// Create plan.md from the bundled template without overwriting work.
    Plan {
        feature: PathBuf,
        #[arg(long)]
        dry_run: bool,
    },
    /// Validate feature artifacts and print their paths as JSON.
    Prerequisites {
        feature: PathBuf,
        #[arg(long)]
        require_tasks: bool,
        #[arg(long)]
        paths_only: bool,
    },
    /// Update the managed plan context in AGENTS.md, preserving user content.
    Context {
        feature: PathBuf,
        #[arg(long)]
        dry_run: bool,
    },
    /// Run a specification workflow instruction through the selected backend.
    Prompt {
        #[arg(value_enum)]
        operation: Operation,
        feature: PathBuf,
        #[arg(long, default_value = "")]
        input: String,
        #[arg(long)]
        dry_run: bool,
    },
    /// Create issues from unchecked task lines, skipping existing task IDs.
    Issues {
        tasks: PathBuf,
        #[arg(long)]
        dry_run: bool,
    },
    /// Create task labels and apply phase/story/priority labels to task issues.
    Labels {
        tasks: PathBuf,
        #[arg(long)]
        dry_run: bool,
    },
    /// Shift task references by an explicit offset; preserve a backup.
    Renumber {
        tasks: PathBuf,
        #[arg(long, allow_hyphen_values = true)]
        offset: i64,
        #[arg(long)]
        dry_run: bool,
    },
    /// Shift task IDs in issue titles by an explicit offset.
    IssueTitles {
        #[arg(long, allow_hyphen_values = true)]
        offset: i64,
        #[arg(long)]
        dry_run: bool,
    },
    /// Run all required checks against the selected checkout.
    Check {
        #[arg(long)]
        dry_run: bool,
    },
    /// Remove a clean published worktree, retaining its branch and logs.
    Cleanup {
        #[arg(long)]
        issue: u64,
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum Operation {
    Specify,
    Clarify,
    Plan,
    Tasks,
    Analyze,
    Checklist,
    Constitution,
    Implement,
    Taskstoissues,
}
impl Operation {
    fn name(self) -> &'static str {
        match self {
            Self::Specify => "specify",
            Self::Clarify => "clarify",
            Self::Plan => "plan",
            Self::Tasks => "tasks",
            Self::Analyze => "analyze",
            Self::Checklist => "checklist",
            Self::Constitution => "constitution",
            Self::Implement => "implement",
            Self::Taskstoissues => "taskstoissues",
        }
    }
}

fn asset(name: &str) -> Result<&'static str> {
    assets::FILES
        .iter()
        .find(|(path, _)| *path == name)
        .map(|(_, text)| *text)
        .ok_or_else(|| format!("missing bundled asset: {name}").into())
}

fn create(path: &Path, text: &str) -> Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(text.as_bytes())?;
    Ok(())
}

pub fn init(root: &Path) -> Result<()> {
    for (name, text) in assets::FILES {
        let path = root.join("workflow").join(name);
        fs::create_dir_all(path.parent().ok_or("asset parent missing")?)?;
        if !path.exists() {
            create(&path, text)?;
        }
    }
    let readme = root.join("workflow/README.md");
    if !readme.exists() {
        create(&readme, include_str!("../workflow/README.md"))?;
    }
    let config = root.join("autodev.json");
    let constitution = root.join("workflow/constitution.md");
    if !constitution.exists() {
        create(&constitution, asset("templates/constitution-template.md")?)?;
    }
    if !config.exists() {
        create(&config, include_str!("../autodev.example.json"))?;
    }
    println!("Workflow installed without overwriting existing files. Configure autodev.json and commit before running.");
    Ok(())
}

#[derive(Debug, Serialize)]
struct Task {
    id: String,
    title: String,
    body: String,
    labels: Vec<String>,
}

fn infer_labels(title: &str, body: &str) -> Vec<String> {
    let text = format!("{title} {body}").to_lowercase();
    let mut labels = std::collections::BTreeSet::new();
    for (label, words) in [
        (
            "backend",
            "backend django api model serializer view celery database migration postgres",
        ),
        (
            "frontend",
            "frontend next.js nextjs react component ui tailwind shadcn",
        ),
        ("api", "endpoint api rest graphql contract"),
        ("ui", "ui button form modal calendar dashboard interface"),
        ("database", "database migration schema postgres sql model"),
        ("devops", "docker ci/cd pipeline deployment"),
    ] {
        if words.split_whitespace().any(|word| text.contains(word)) {
            labels.insert(label.into());
        }
    }
    if text.contains("github actions") {
        labels.insert("devops".into());
    }
    for (marker, prefix) in [
        ("**phase**:", "phase-"),
        ("**priority**:", ""),
        ("**user story**:", ""),
    ] {
        if let Some((_, rest)) = text.split_once(marker) {
            if let Some(value) = rest.split_whitespace().next() {
                let value = value.trim_matches(['[', ']']);
                if value.chars().all(|c| c.is_ascii_alphanumeric()) {
                    labels.insert(format!("{prefix}{value}"));
                }
            }
        }
    }
    for rest in text.split("[us").skip(1) {
        if let Some((number, _)) = rest.split_once(']') {
            if !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()) {
                labels.insert(format!("us{number}"));
            }
        }
    }
    labels.into_iter().collect()
}

fn ensure_labels(root: &Path, config: &Config, labels: &[String]) -> Result<()> {
    let pages: Vec<Vec<serde_json::Value>> = serde_json::from_str(&output(
        root,
        "gh",
        &[
            "api",
            "--paginate",
            "--slurp",
            &format!("repos/{}/labels?per_page=100", config.github_repo),
        ],
    )?)?;
    let mut existing: std::collections::HashSet<String> = pages
        .into_iter()
        .flatten()
        .filter_map(|v| v["name"].as_str().map(str::to_owned))
        .collect();
    for label in labels {
        if existing.insert(label.clone()) {
            output(
                root,
                "gh",
                &[
                    "label",
                    "create",
                    label,
                    "--repo",
                    &config.github_repo,
                    "--color",
                    "607D8B",
                ],
            )?;
        }
    }
    Ok(())
}

fn tasks(text: &str) -> Result<Vec<Task>> {
    let mut result = Vec::new();
    let mut phase = String::new();
    let mut goal = String::new();
    let mut acceptance = String::new();
    let mut ids = std::collections::HashSet::new();
    for line in text.lines().map(str::trim) {
        if let Some(value) = line.strip_prefix("## Phase ") {
            phase = value.into();
            goal.clear();
            acceptance.clear();
        }
        if let Some(value) = line.strip_prefix("**Goal**:") {
            goal = value.trim().into();
        }
        if let Some(value) = line.strip_prefix("**Independent Test**:") {
            acceptance = value.trim().into();
        }
        let Some(task) = line.strip_prefix("- [ ]").map(str::trim) else {
            continue;
        };
        let (id, mut description) = task
            .split_once(char::is_whitespace)
            .ok_or("task requires ID and description")?;
        if !id.starts_with('T')
            || id.len() < 2
            || !id[1..].bytes().all(|c| c.is_ascii_digit())
            || !ids.insert(id.to_owned())
        {
            return Err(format!("invalid or duplicate task ID: {id}").into());
        }
        let mut labels = Vec::new();
        let mut metadata = String::new();
        description = description.trim();
        while description.starts_with('[') {
            let end = description.find(']').ok_or("unclosed task metadata")?;
            let tag = &description[1..end];
            if tag == "P" {
                labels.push("parallel".into());
                metadata.push_str("**Parallelizable**: Yes\n");
            } else if tag.starts_with("US")
                && tag[2..].bytes().all(|c| c.is_ascii_digit())
                && tag.len() > 2
            {
                labels.push(tag.to_lowercase());
                metadata.push_str(&format!("**User Story**: {tag}\n"));
            } else {
                break;
            }
            description = description[end + 1..].trim();
        }
        if description.is_empty() {
            return Err("empty task description".into());
        }
        if let Some(number) = phase
            .split(':')
            .next()
            .filter(|v| !v.is_empty() && v.bytes().all(|c| c.is_ascii_digit()))
        {
            labels.push(format!("phase-{number}"));
        }
        for priority in ["P0", "P1", "P2", "P3"] {
            if phase.contains(&format!("Priority: {priority}")) {
                labels.push(priority.to_lowercase());
            }
        }
        labels.extend(infer_labels(description, ""));
        labels.sort();
        labels.dedup();
        result.push(Task { id: id.into(), title: format!("[{id}] {description}"), body: format!("## Task: {id}\n\n**Phase**: {phase}\n{metadata}\n### Description\n{description}\n\n### Phase Goal\n{goal}\n\n### Acceptance Criteria\n{acceptance}"), labels });
    }
    Ok(result)
}

fn renumber(text: &str, offset: i64) -> Result<String> {
    let mut result = String::new();
    let mut chars = text.char_indices().peekable();
    let mut previous = None;
    while let Some((_, c)) = chars.next() {
        if c == 'T' && previous.is_none_or(|p: char| !p.is_alphanumeric() && p != '_') {
            let mut digits = String::new();
            while chars.peek().is_some_and(|(_, ch)| ch.is_ascii_digit()) {
                digits.push(chars.next().unwrap().1);
            }
            let boundary = chars
                .peek()
                .is_none_or(|(_, ch)| !ch.is_alphanumeric() && *ch != '_');
            if !digits.is_empty() && boundary {
                let value = digits
                    .parse::<i64>()?
                    .checked_add(offset)
                    .filter(|v| *v > 0)
                    .ok_or("task ID offset out of range")?;
                result.push_str(&format!("T{value:03}"));
            } else {
                result.push('T');
                result.push_str(&digits);
            }
            previous = Some(if digits.is_empty() { 'T' } else { '0' });
        } else {
            result.push(c);
            previous = Some(c);
        }
    }
    Ok(result)
}

fn feature_path(root: &Path, path: &Path) -> Result<PathBuf> {
    let canonical = root.join(path).canonicalize()?;
    if !canonical.starts_with(root.canonicalize()?) {
        return Err("feature must be inside repository".into());
    }
    Ok(canonical)
}

pub fn execute_action(root: &Path, action: Action) -> Result<()> {
    let label_only = matches!(action, Action::Labels { .. });
    match action {
        Action::Feature {
            name,
            number,
            dry_run,
        } => {
            let slug = name
                .split(|c: char| !c.is_ascii_alphanumeric())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("-")
                .to_lowercase();
            if slug.is_empty() {
                return Err("feature name must contain ASCII letters or digits".into());
            }
            let specs = root.join("specs");
            let mut maximum = 0;
            if specs.exists() {
                for entry in fs::read_dir(&specs)? {
                    let name = entry?.file_name().to_string_lossy().into_owned();
                    maximum = maximum.max(
                        name.split('-')
                            .next()
                            .and_then(|s| s.parse::<u64>().ok())
                            .unwrap_or(0),
                    );
                }
            }
            if !dry_run && number.is_none() {
                let refs = git(
                    root,
                    &[
                        "for-each-ref",
                        "--format=%(refname:short)",
                        "refs/heads",
                        "refs/remotes",
                    ],
                )?;
                let remote = if git(root, &["remote"])?.lines().any(|r| r == "origin") {
                    git(root, &["ls-remote", "--heads", "origin"])?
                } else {
                    String::new()
                };
                for line in refs.lines().chain(remote.lines()) {
                    let branch = line
                        .split_whitespace()
                        .last()
                        .unwrap_or("")
                        .rsplit('/')
                        .next()
                        .unwrap_or("");
                    maximum = maximum.max(
                        branch
                            .split('-')
                            .next()
                            .and_then(|n| n.parse().ok())
                            .unwrap_or(0),
                    );
                }
            }
            let number = number.unwrap_or(maximum.checked_add(1).ok_or("feature number overflow")?);
            if number == 0 {
                return Err("feature number must be positive".into());
            }
            let branch = format!("{number:03}-{slug}");
            let path = specs.join(&branch);
            if path.exists() {
                return Err("feature already exists; work preserved".into());
            }
            if dry_run {
                println!("Would create branch {branch} and {}", path.display());
                return Ok(());
            }
            if !git(root, &["status", "--porcelain"])?.is_empty() {
                return Err("working tree must be clean".into());
            }
            git(root, &["check-ref-format", "--branch", &branch])?;
            git(root, &["checkout", "-b", &branch])?;
            fs::create_dir_all(&path)?;
            create(&path.join("spec.md"), asset("templates/spec-template.md")?)?;
            println!(
                "{}",
                serde_json::json!({"branch":branch,"feature_dir":path,"spec":path.join("spec.md")})
            );
        }
        Action::Plan { feature, dry_run } => {
            let feature = feature_path(root, &feature)?;
            if !feature.join("spec.md").is_file() {
                return Err("spec.md required".into());
            }
            if dry_run {
                println!("Would create {}", feature.join("plan.md").display());
            } else {
                create(
                    &feature.join("plan.md"),
                    asset("templates/plan-template.md")?,
                )?;
                println!(
                    "{}",
                    serde_json::json!({"FEATURE_SPEC":feature.join("spec.md"),"IMPL_PLAN":feature.join("plan.md"),"SPECS_DIR":feature})
                );
            }
        }
        Action::Prerequisites {
            feature,
            require_tasks,
            paths_only,
        } => {
            let feature = feature_path(root, &feature)?;
            for file in ["spec.md", "plan.md"]
                .into_iter()
                .chain(require_tasks.then_some("tasks.md"))
            {
                if !paths_only && !feature.join(file).is_file() {
                    return Err(format!("required artifact missing: {file}").into());
                }
            }
            let documents: Vec<_> = [
                "spec.md",
                "plan.md",
                "tasks.md",
                "research.md",
                "data-model.md",
                "contracts",
                "quickstart.md",
                "checklists",
            ]
            .into_iter()
            .filter(|name| feature.join(name).exists())
            .collect();
            println!(
                "{}",
                serde_json::json!({"FEATURE_DIR":feature,"AVAILABLE_DOCS":documents,"FEATURE_SPEC":feature.join("spec.md"),"IMPL_PLAN":feature.join("plan.md"),"TASKS":feature.join("tasks.md")})
            );
        }
        Action::Context { feature, dry_run } => {
            let feature = feature_path(root, &feature)?;
            let plan = fs::read_to_string(feature.join("plan.md"))?;
            let path = root.join("AGENTS.md");
            let old = if path.exists() {
                fs::read_to_string(&path)?
            } else {
                String::new()
            };
            let start = "<!-- autodev:plan:start -->";
            let end = "<!-- autodev:plan:end -->";
            let block = format!("{start}\n# Current feature plan\n\n{plan}\n{end}");
            let updated = match (old.find(start), old.find(end)) {
                (Some(a), Some(b)) if a < b => {
                    format!("{}{}{}", &old[..a], block, &old[b + end.len()..])
                }
                (None, None) => format!("{old}\n\n{block}\n"),
                _ => {
                    return Err(
                        "invalid managed context markers; preserve and repair manually".into(),
                    )
                }
            };
            if dry_run {
                println!("{updated}");
            } else {
                fs::write(path, updated)?;
            }
        }
        Action::Prompt {
            operation,
            feature,
            input,
            dry_run,
        } => {
            let config = Config::load(root)?;
            let name = operation.name();
            if dry_run {
                println!(
                    "Would execute {name} for {} through {:?}",
                    feature.display(),
                    config.agent
                );
                return Ok(());
            }
            let feature = feature_path(root, &feature)?;
            let path = root.join(format!("workflow/commands/{name}.md"));
            let instructions = if path.exists() {
                fs::read_to_string(path)?
            } else {
                asset(&format!("commands/{name}.md"))?.into()
            };
            let prompt = format!("Workflow operation: {name}\nFeature directory: {}\nUser input: {input}\n\n{instructions}\n\nDo not commit, push, merge, change branches, or create issues/PRs. Use project issues for reviewed task publication. Write autodev-stage-result.json at the repository root with {{\"status\":\"passed\",\"blockers\":[]}} or status blocked with blocker strings. Missing answers to clarification questions are blockers; do not invent user answers.", feature.display());
            let logs = state_dir(root)?.join("project");
            fs::create_dir_all(&logs)?;
            let dir = state_dir(root)?;
            let _lock = Lock::acquire(&dir)?;
            let result_path = root.join("autodev-stage-result.json");
            if result_path.exists() {
                return Err("stage result already exists; inspect before retrying".into());
            }
            let head = git(root, &["rev-parse", "HEAD"])?;
            let branch = git(root, &["branch", "--show-current"])?;
            let input_path = logs.join(format!("{name}-prompt.txt"));
            fs::write(&input_path, &prompt)?;
            let mut argv = config.agent.clone();
            let stdin = if config.agent_input == AgentInput::Stdin {
                Some(input_path.as_path())
            } else {
                argv.push(prompt);
                None
            };
            execute(
                root,
                &argv,
                &logs.join(format!("{name}.log")),
                config.timeout_seconds,
                &CANCELLED,
                stdin,
            )?;
            if git(root, &["rev-parse", "HEAD"])? != head
                || git(root, &["branch", "--show-current"])? != branch
            {
                return Err("agent changed branch/HEAD during project operation".into());
            }
            workflow::stage_result(&result_path, &logs.join(format!("{name}-result.json")))?;
        }
        Action::Issues {
            tasks: path,
            dry_run,
        }
        | Action::Labels {
            tasks: path,
            dry_run,
        } => {
            let parsed = tasks(&fs::read_to_string(root.join(path))?)?;
            if dry_run {
                println!("{}", serde_json::to_string_pretty(&parsed)?);
                return Ok(());
            }
            let config = Config::load(root)?;
            check_remote(root, &config.github_repo)?;
            let existing = workflow::issues(root, &config)?;
            let mut labels = std::collections::BTreeSet::new();
            for task in &parsed {
                labels.extend(task.labels.iter());
            }
            ensure_labels(
                root,
                &config,
                &labels.into_iter().cloned().collect::<Vec<_>>(),
            )?;
            for task in parsed {
                let prefix = format!("[{}] ", task.id);
                let matching: Vec<_> = existing
                    .iter()
                    .filter(|issue| issue.title.starts_with(&prefix))
                    .collect();
                if matching.len() > 1 {
                    return Err(format!("multiple issues match {}", task.id).into());
                }
                if let Some(issue) = matching.first() {
                    if label_only {
                        for label in &task.labels {
                            output(
                                root,
                                "gh",
                                &[
                                    "issue",
                                    "edit",
                                    &issue.number.to_string(),
                                    "--repo",
                                    &config.github_repo,
                                    "--add-label",
                                    label,
                                ],
                            )?;
                        }
                    }
                    println!("Existing task {}: #{}", task.id, issue.number);
                } else if !label_only {
                    let mut args = vec![
                        "issue",
                        "create",
                        "--repo",
                        &config.github_repo,
                        "--title",
                        &task.title,
                        "--body",
                        &task.body,
                    ];
                    for label in &task.labels {
                        args.extend(["--label", label]);
                    }
                    println!("{}", output(root, "gh", &args)?);
                }
            }
        }
        Action::Renumber {
            tasks: path,
            offset,
            dry_run,
        } => {
            let path = root.join(path);
            let original = fs::read_to_string(&path)?;
            let updated = renumber(&original, offset)?;
            if dry_run {
                println!("{updated}");
            } else {
                create(&path.with_extension("md.backup"), &original)?;
                fs::write(path, updated)?;
            }
        }
        Action::IssueTitles { offset, dry_run } => {
            let config = Config::load(root)?;
            if dry_run {
                println!("Would shift task IDs in issue titles by {offset}; no API calls made.");
                return Ok(());
            }
            check_remote(root, &config.github_repo)?;
            for issue in workflow::issues(root, &config)? {
                if issue.title.starts_with("[T") {
                    let end = issue.title.find(']').ok_or("invalid task issue title")?;
                    let title = format!(
                        "{}{}",
                        renumber(&issue.title[..=end], offset)?,
                        &issue.title[end + 1..]
                    );
                    output(
                        root,
                        "gh",
                        &[
                            "issue",
                            "edit",
                            &issue.number.to_string(),
                            "--repo",
                            &config.github_repo,
                            "--title",
                            &title,
                        ],
                    )?;
                }
            }
        }
        Action::IssueLabels { dry_run } => {
            if dry_run {
                println!("Would infer technical and task metadata labels from open issues. No API calls made.");
                return Ok(());
            }
            let config = Config::load(root)?;
            check_remote(root, &config.github_repo)?;
            for issue in workflow::issues(root, &config)?
                .iter()
                .filter(|i| i.state.eq_ignore_ascii_case("open"))
            {
                let labels = infer_labels(&issue.title, issue.body.as_deref().unwrap_or(""));
                ensure_labels(root, &config, &labels)?;
                for label in labels {
                    output(
                        root,
                        "gh",
                        &[
                            "issue",
                            "edit",
                            &issue.number.to_string(),
                            "--repo",
                            &config.github_repo,
                            "--add-label",
                            &label,
                        ],
                    )?;
                }
            }
        }
        Action::Check { dry_run } => {
            let config = Config::load(root)?;
            if dry_run {
                println!("Would run {:?}", config.checks);
                return Ok(());
            }
            let dir = state_dir(root)?;
            fs::create_dir_all(&dir)?;
            let _lock = Lock::acquire(&dir)?;
            let logs = dir.join("project");
            fs::create_dir_all(&logs)?;
            let index = logs.join("check-index");
            if index.exists() {
                return Err(
                    "previous temporary check index exists; inspect before retrying".into(),
                );
            }
            let result = (|| {
                output_with_index(root, "git", &["read-tree", "HEAD"], Some(&index))?;
                workflow::checks(root, &config, &logs, Some(&index))
            })();
            if index.exists() {
                fs::remove_file(index)?;
            }
            result?;
        }
        Action::Cleanup { issue, dry_run } => {
            let config = Config::load(root)?;
            if dry_run {
                println!("Would safely remove published worktree for #{issue}");
            } else {
                workflow::cleanup(root, &config, issue, false)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn task_metadata_and_simultaneous_renumbering() {
        let text = "## Phase 2: Login (Priority: P1)\n**Goal**: Secure login\n**Independent Test**: Reject invalid token\n- [ ] T001 [P] [US1] Implement login\n- [x] T002 Done\n";
        let parsed = tasks(text).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].labels, ["p1", "parallel", "phase-2", "us1"]);
        assert!(parsed[0].body.contains("Reject invalid token"));
        assert!(tasks("- [ ] T001 One\n- [ ] T001 Two").is_err());
        assert_eq!(
            renumber("T001 -> T003; AT002; T003x; T999", 2).unwrap(),
            "T003 -> T005; AT002; T003x; T1001"
        );
        assert!(renumber("T001", -1).is_err());
    }
}
