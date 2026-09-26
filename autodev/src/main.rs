use clap::{Parser, Subcommand};
mod assets;
mod project;
mod workflow;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
static CANCELLED: AtomicBool = AtomicBool::new(false);

#[derive(Parser)]
#[command(version, about = "Supervised issue-to-PR development")]
struct Cli {
    #[arg(long, global = true, default_value = ".")]
    repo: PathBuf,
    #[command(subcommand)]
    command: Action,
}

#[derive(Subcommand)]
enum Action {
    /// Check repository, configuration, and installed tools.
    Doctor,
    /// Show the last persisted cycle event.
    Status,
    /// Select the next sequential open issue.
    Next,
    /// Install provider-neutral workflow instructions and templates.
    Init,
    /// Repeat cycles, waiting for each exact PR to be manually merged.
    Loop {
        #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u64).range(1..))]
        poll_interval: u64,
        #[arg(long, default_value_t = 0)]
        max_cycles: u64,
        #[arg(long)]
        wait_for_merge: Option<u64>,
        #[arg(long)]
        dry_run: bool,
    },
    /// Run a project preparation or issue-maintenance operation.
    Project {
        #[command(subcommand)]
        action: project::Action,
    },
    /// Implement one open issue, validate, and optionally publish a draft PR.
    Run {
        #[arg(long, value_parser = clap::value_parser!(u64).range(1..))]
        issue: Option<u64>,
        /// Continue the saved cycle, preserving completed stages and work.
        #[arg(long)]
        resume: bool,
        /// Stop after prepare, implement, validate, commit, push, or pr.
        #[arg(long, value_enum)]
        until: Option<workflow::Stop>,
        /// Print the plan without network requests, processes, or file writes.
        #[arg(long)]
        dry_run: bool,
        /// Push the validated commit and create a draft PR. Never merge.
        #[arg(long)]
        publish: bool,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    base_branch: String,
    /// Explicit GitHub owner/repo: avoids relying on gh's current directory.
    github_repo: String,
    agent: Vec<String>,
    #[serde(default)]
    agent_input: AgentInput,
    #[serde(default)]
    agent_probe: Option<Vec<String>>,
    checks: Vec<Vec<String>>,
    #[serde(default = "default_timeout")]
    timeout_seconds: u64,
    #[serde(default)]
    workflow: workflow::Settings,
}

#[derive(Deserialize, Default, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
enum AgentInput {
    #[default]
    Argument,
    Stdin,
}

fn default_timeout() -> u64 {
    3600
}

impl Config {
    fn load(root: &Path) -> Result<Self> {
        let bytes = fs::read(root.join("autodev.json"))?;
        let config: Self =
            serde_json::from_slice(bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(&bytes))?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        if self.base_branch.is_empty() || self.base_branch.starts_with('-') {
            return Err("base_branch must be a nonempty branch name".into());
        }
        let parts: Vec<_> = self.github_repo.split('/').collect();
        if parts.len() != 2
            || parts.iter().any(|part| {
                part.is_empty()
                    || !part
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
            })
        {
            return Err("github_repo must be owner/repository".into());
        }
        if self.timeout_seconds == 0 || self.checks.is_empty() {
            return Err("at least one check and a positive timeout are required".into());
        }
        for argv in std::iter::once(&self.agent)
            .chain(self.agent_probe.iter())
            .chain(self.checks.iter())
        {
            if argv.is_empty() || argv[0].trim().is_empty() || argv.iter().any(|s| s.contains('\0'))
            {
                return Err("commands must be nonempty argument arrays without NUL bytes".into());
            }
        }
        self.workflow.validate()?;
        Ok(())
    }
}

fn output(root: &Path, program: &str, args: &[&str]) -> Result<String> {
    output_with_index(root, program, args, None)
}

fn output_with_index(
    root: &Path,
    program: &str,
    args: &[&str],
    index: Option<&Path>,
) -> Result<String> {
    if CANCELLED.load(Ordering::SeqCst) {
        return Err("cancelled before starting next command".into());
    }
    let mut command = Command::new(program);
    if let Some(index) = index {
        command.env("GIT_INDEX_FILE", index);
    }
    let output = command
        .args(args)
        .current_dir(root)
        .stdin(Stdio::null())
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "{program} {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn git(root: &Path, args: &[&str]) -> Result<String> {
    output(root, "git", args)
}

fn check_remote(root: &Path, expected: &str) -> Result<()> {
    let allowed = [
        format!("https://github.com/{expected}"),
        format!("https://github.com/{expected}.git"),
        format!("git@github.com:{expected}"),
        format!("git@github.com:{expected}.git"),
    ];
    for args in [
        vec!["remote", "get-url", "--all", "origin"],
        vec!["remote", "get-url", "--push", "--all", "origin"],
    ] {
        let urls = git(root, &args)?;
        if urls.is_empty()
            || urls
                .lines()
                .any(|url| !allowed.iter().any(|item| item == url))
        {
            return Err(
                "all origin fetch/push URLs must match github_repo (GitHub HTTPS or SSH URL)"
                    .into(),
            );
        }
    }
    Ok(())
}

fn repository(path: &Path) -> Result<PathBuf> {
    Ok(PathBuf::from(git(path, &["rev-parse", "--show-toplevel"])?))
}

fn state_dir(root: &Path) -> Result<PathBuf> {
    Ok(PathBuf::from(git(
        root,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?)
    .join("autodev"))
}

#[derive(Serialize, Deserialize, Debug)]
struct Event {
    issue: u64,
    phase: String,
    worktree: PathBuf,
    detail: String,
}

fn record(dir: &Path, issue: u64, worktree: &Path, phase: &str, detail: &str) -> Result<()> {
    let event = Event {
        issue,
        phase: phase.into(),
        worktree: worktree.into(),
        detail: detail.into(),
    };
    let mut bytes = serde_json::to_vec(&event)?;
    bytes.push(b'\n');
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("events.jsonl"))?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    println!("{phase}: {detail}");
    Ok(())
}

struct Lock(PathBuf);
impl Lock {
    fn acquire(dir: &Path) -> Result<Self> {
        let path = dir.join("run.lock");
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| {
                format!(
                    "cannot acquire {}: {e}; if stale, verify no run is active before removing it",
                    path.display()
                )
            })?;
        let lock = Self(path);
        writeln!(file, "{}", std::process::id())?;
        file.sync_all()?;
        Ok(lock)
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn terminate(child: &mut std::process::Child) {
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &child.id().to_string(), "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    #[cfg(unix)]
    {
        let _ = Command::new("kill")
            .args(["-KILL", "--", &format!("-{}", child.id())])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
    let _ = child.wait();
}

fn execute(
    root: &Path,
    argv: &[String],
    log: &Path,
    timeout: u64,
    cancelled: &AtomicBool,
    input: Option<&Path>,
) -> Result<()> {
    if cancelled.load(Ordering::SeqCst) {
        return Err("cancelled".into());
    }
    let file = File::create(log)?;
    let mut command = Command::new(&argv[0]);
    command
        .args(&argv[1..])
        .current_dir(root)
        .stdin(match input {
            Some(path) => Stdio::from(File::open(path)?),
            None => Stdio::null(),
        })
        .stdout(Stdio::from(file.try_clone()?))
        .stderr(Stdio::from(file));
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command.spawn()?;
    let start = Instant::now();
    loop {
        if cancelled.load(Ordering::SeqCst) || start.elapsed() >= Duration::from_secs(timeout) {
            terminate(&mut child);
            return Err(format!(
                "cancelled or timed out: {}; log: {}",
                argv[0],
                log.display()
            )
            .into());
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return Err(
                        format!("{} exited {status}; log: {}", argv[0], log.display()).into(),
                    );
                }
                return Ok(());
            }
            Ok(None) => thread::sleep(Duration::from_millis(100)),
            Err(error) => {
                terminate(&mut child);
                return Err(error.into());
            }
        }
    }
}

#[derive(Deserialize)]
struct Issue {
    title: String,
    body: String,
    state: String,
}

fn main() {
    if let Err(error) = entry() {
        eprintln!("autodev: {error}");
        std::process::exit(1);
    }
}

fn entry() -> Result<()> {
    let cli = Cli::parse();
    if matches!(cli.command, Action::Init) {
        return project::init(&cli.repo);
    }
    if let Action::Project { action } = cli.command {
        ctrlc::set_handler(|| CANCELLED.store(true, Ordering::SeqCst))?;
        return project::execute_action(&cli.repo, action);
    }
    if let Action::Loop {
        dry_run: true,
        poll_interval,
        max_cycles,
        wait_for_merge,
    } = cli.command
    {
        Config::load(&cli.repo)?;
        println!("Would run sequential cycles, poll every {poll_interval}s, maximum {max_cycles} (0 = unlimited), initial PR {wait_for_merge:?}. No actions executed.");
        return Ok(());
    }
    // Dry-run reads only the explicitly selected configuration: no git, gh, or agent.
    if let Action::Run {
        issue,
        dry_run: true,
        publish,
        until,
        resume,
    } = cli.command
    {
        let config = Config::load(&cli.repo)?;
        let stop = workflow::stop(&config, publish, until);
        println!("Issue {issue:?} (None = next) in {}; base {}; agent {:?}; {} required checks; stop: {stop:?}; resume: {resume}; push: {}; PR: {}. No actions executed.", config.github_repo, config.base_branch, config.agent, config.checks.len(), stop >= workflow::Stop::Push, stop == workflow::Stop::Pr);
        return Ok(());
    }
    let root = repository(&cli.repo)?;
    if matches!(cli.command, Action::Status) {
        let path = state_dir(&root)?.join("events.jsonl");
        if !path.exists() {
            println!("No recorded cycles.");
            return Ok(());
        }
        let text = fs::read_to_string(path)?;
        let last = text.lines().last().ok_or("empty state journal")?;
        let event: Event = serde_json::from_str(last)
            .map_err(|e| format!("state journal incomplete or corrupt: {e}"))?;
        println!("{}", serde_json::to_string_pretty(&event)?);
        return Ok(());
    }
    let config = Config::load(&root)?;
    if matches!(cli.command, Action::Doctor) {
        git(
            &root,
            &["check-ref-format", "--branch", &config.base_branch],
        )?;
        println!("{}", output(&root, "git", &["--version"])?);
        println!("{}", output(&root, "gh", &["--version"])?);
        output(&root, "gh", &["auth", "status"])?;
        if let Some(probe) = &config.agent_probe {
            let args: Vec<_> = probe[1..].iter().map(String::as_str).collect();
            println!("{}", output(&root, &probe[0], &args)?);
        } else {
            println!("Agent not probed: configure agent_probe for a backend-specific diagnostic.");
        }
        println!(
            "Configuration valid; {} required checks. Validation checks were not executed.",
            config.checks.len()
        );
        return Ok(());
    }
    ctrlc::set_handler(|| CANCELLED.store(true, Ordering::SeqCst))?;
    match cli.command {
        Action::Next => {
            let items = workflow::issues(&root, &config)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&workflow::select_next(&items))?
            );
        }
        Action::Loop {
            poll_interval,
            max_cycles,
            wait_for_merge,
            ..
        } => workflow::continuous(&root, &config, poll_interval, max_cycles, wait_for_merge)?,
        Action::Run {
            issue,
            publish,
            resume,
            until,
            ..
        } => {
            let issue = match issue {
                Some(number) => number,
                None => {
                    let items = workflow::issues(&root, &config)?;
                    let Some(next) = workflow::select_next(&items) else {
                        println!("All issues completed.");
                        return Ok(());
                    };
                    next.number
                }
            };
            let stop = workflow::stop(&config, publish, until);
            workflow::run(&root, &config, issue, stop, resume, false)?;
        }
        _ => (),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configuration_rejects_missing_checks_and_invalid_commands() {
        let mut config = Config {
            base_branch: "development".into(),
            github_repo: "owner/repo".into(),
            agent: vec!["claude".into(), "-p".into()],
            agent_input: AgentInput::Argument,
            agent_probe: None,
            checks: vec![vec!["cargo".into(), "test".into()]],
            timeout_seconds: 1,
            workflow: workflow::Settings::default(),
        };
        assert!(config.validate().is_ok());
        config.checks.clear();
        assert!(config.validate().is_err());
        config.checks.push(vec![]);
        assert!(config.validate().is_err());
    }

    #[test]
    fn config_defaults_and_trust_boundary_validation() {
        let valid = serde_json::json!({"base_branch":"main", "github_repo":"owner/repo", "agent":["agent", "argument with spaces"], "checks":[["test"]]});
        let parsed: Config = serde_json::from_value(valid.clone()).unwrap();
        assert_eq!(parsed.timeout_seconds, 3600);
        assert!(parsed.validate().is_ok());
        let dir = std::env::temp_dir().join(format!("autodev-bom-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("autodev.json"), format!("\u{feff}{valid}")).unwrap();
        assert_eq!(Config::load(&dir).unwrap().github_repo, "owner/repo");
        fs::remove_dir_all(dir).unwrap();
        for (key, value) in [
            ("github_repo", serde_json::json!("owner/repo/extra")),
            ("base_branch", serde_json::json!("--upload-pack=evil")),
            ("timeout_seconds", serde_json::json!(0)),
            ("agent", serde_json::json!([])),
            ("agent_probe", serde_json::json!([])),
            ("agent_probe", serde_json::json!(["bad\u{0}probe"])),
            ("checks", serde_json::json!([["test", "bad\u{0}arg"]])),
        ] {
            let mut bad = valid.clone();
            bad[key] = value;
            assert!(
                serde_json::from_value::<Config>(bad)
                    .unwrap()
                    .validate()
                    .is_err(),
                "{key}"
            );
        }
        let mut unknown = valid.clone();
        unknown["agent_input"] = serde_json::json!("unsupported");
        assert!(serde_json::from_value::<Config>(unknown).is_err());
        assert_eq!(
            serde_json::from_value::<Config>(valid.clone())
                .unwrap()
                .agent_input,
            AgentInput::Argument
        );
        let mut unknown = valid;
        unknown["auto_merge"] = serde_json::json!(true);
        assert!(serde_json::from_value::<Config>(unknown).is_err());
    }

    #[test]
    fn already_cancelled_execution_creates_no_log_or_process() {
        let log = std::env::temp_dir().join(format!("autodev-cancel-{}.log", std::process::id()));
        let result = execute(
            Path::new("."),
            &["nonexistent-autodev-command".into()],
            &log,
            1,
            &AtomicBool::new(true),
            None,
        );
        assert_eq!(result.unwrap_err().to_string(), "cancelled");
        assert!(!log.exists());
    }

    #[test]
    fn journal_appends_complete_events() {
        let dir = std::env::temp_dir().join(format!("autodev-journal-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        record(
            &dir,
            7,
            Path::new("work tree"),
            "starting",
            "line one\nline two",
        )
        .unwrap();
        record(&dir, 7, Path::new("work tree"), "failed", "preserved").unwrap();
        let text = fs::read_to_string(dir.join("events.jsonl")).unwrap();
        let events: Vec<Event> = text
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].detail, "line one\nline two");
        assert_eq!(events[1].phase, "failed");
        assert_eq!(events[1].issue, 7);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn real_git_worktree_and_lock_preserve_original_checkout() {
        let root = std::env::temp_dir().join(format!("autodev-real-git-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        git(&root, &["init", "-b", "development"]).unwrap();
        git(
            &root,
            &[
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
                "commit",
                "--allow-empty",
                "-m",
                "initial",
            ],
        )
        .unwrap();
        let dir = state_dir(&root).unwrap();
        fs::create_dir_all(&dir).unwrap();
        let lock = Lock::acquire(&dir).unwrap();
        assert!(Lock::acquire(&dir).is_err());
        let worktree = dir.join("worktrees/issue-7");
        git(
            &root,
            &[
                "worktree",
                "add",
                "-b",
                "autodev/issue-7",
                &worktree.to_string_lossy(),
                "HEAD",
            ],
        )
        .unwrap();
        fs::write(worktree.join("change.txt"), "preserved").unwrap();
        assert_eq!(
            git(&root, &["branch", "--show-current"]).unwrap(),
            "development"
        );
        assert!(git(&root, &["status", "--porcelain"]).unwrap().is_empty());
        assert_eq!(state_dir(&worktree).unwrap(), dir);
        drop(lock);
        assert!(!dir.join("run.lock").exists());
        fs::remove_dir_all(root).unwrap();
    }
}
