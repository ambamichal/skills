use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::OnceLock,
};

fn fixture_binary() -> &'static PathBuf {
    static BINARY: OnceLock<PathBuf> = OnceLock::new();
    BINARY.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("autodev-fixture-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let binary = dir.join(format!("fixture{}", std::env::consts::EXE_SUFFIX));
        assert!(Command::new("rustc")
            .args(["--crate-name", "fixture", "tests/fixture.rs.txt", "-o"])
            .arg(&binary)
            .status()
            .unwrap()
            .success());
        binary
    })
}

struct Sandbox(PathBuf, Option<PathBuf>);
impl Sandbox {
    fn configure(&self, change: impl FnOnce(&mut serde_json::Value)) {
        let path = self.0.join("autodev.json");
        let mut value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        change(&mut value);
        fs::write(path, value.to_string()).unwrap();
    }
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("autodev-test-{}-{name}", std::process::id()));
        fs::create_dir_all(dir.join("bin")).unwrap();
        for tool in ["git", "gh", "worker"] {
            fs::copy(
                fixture_binary(),
                dir.join("bin")
                    .join(format!("{tool}{}", std::env::consts::EXE_SUFFIX)),
            )
            .unwrap();
        }
        let config = serde_json::json!({"base_branch":"development", "github_repo":"test/project", "agent":["worker","agent"], "checks":[["worker","check"]], "timeout_seconds":1});
        fs::write(dir.join("autodev.json"), config.to_string()).unwrap();
        Self(dir, None)
    }
    fn invoke(&self, args: &[&str], mode: &str) -> Output {
        self.command(args, mode).output().unwrap()
    }
    fn command(&self, args: &[&str], mode: &str) -> Command {
        let mut paths = vec![self.0.join("bin")];
        paths.extend(std::env::split_paths(
            &std::env::var_os("PATH").unwrap_or_default(),
        ));
        let mut command = Command::new(env!("CARGO_BIN_EXE_autodev"));
        command
            .args(["--repo"])
            .arg(&self.0)
            .args(args)
            .env("PATH", std::env::join_paths(paths).unwrap())
            .env("AUTODEV_TEST_DIR", &self.0)
            .env("AUTODEV_TEST_MODE", mode)
            .env(
                "AUTODEV_REAL_GIT",
                self.1.as_deref().unwrap_or(std::path::Path::new("")),
            );
        command
    }
    fn calls(&self) -> String {
        fs::read_to_string(self.0.join("calls")).unwrap_or_default()
    }
}

#[test]
fn staged_workflow_blocks_and_resumes_without_repeating_completed_agents() {
    let test = Sandbox::new("stages");
    fs::write(
        test.0.join("stage.md"),
        "Implement the selected task and report blockers.",
    )
    .unwrap();
    test.configure(|v| v["workflow"] = serde_json::json!({"stages":[{"name":"discovery","prompt":"stage.md"},{"name":"quality","prompt":"stage.md"}]}));
    let failed = test.invoke(&["run", "--issue", "7", "--publish"], "stage-blocked");
    assert!(!failed.status.success(), "{failed:?}");
    assert!(!test.calls().contains("\"commit\""));
    let validated = test.invoke(
        &["run", "--issue", "7", "--resume", "--until", "validate"],
        "",
    );
    assert!(validated.status.success(), "{validated:?}");
    assert_eq!(test.calls().matches("worker [\"agent\"").count(), 3);
    let published = test.invoke(&["run", "--issue", "7", "--resume", "--publish"], "");
    assert!(published.status.success(), "{published:?}");
    assert_eq!(test.calls().matches("worker [\"agent\"").count(), 3);
    assert_eq!(test.calls().matches("gh [\"pr\", \"create\"").count(), 1);
    let resumed = test.invoke(&["run", "--issue", "7", "--resume", "--publish"], "");
    assert!(resumed.status.success(), "{resumed:?}");
    assert_eq!(test.calls().matches("gh [\"pr\", \"create\"").count(), 1);
}

#[test]
fn continuous_loop_waits_for_exact_pr_and_recovers_pending_cycle() {
    for mode in ["pr-open-once", "pr-closed"] {
        let test = Sandbox::new(mode);
        let result = test.invoke(&["loop", "--max-cycles", "1", "--poll-interval", "1"], mode);
        assert_eq!(result.status.success(), mode != "pr-closed", "{result:?}");
        assert!(test
            .calls()
            .contains("\"view\", \"https://github.com/test/project/pull/1\""));
        assert!(!test.calls().contains("\"merge\""));
        if mode == "pr-closed" {
            assert!(test.0.join(".git/autodev/loop.json").exists());
            let resumed = test.invoke(
                &["loop", "--max-cycles", "1", "--poll-interval", "1"],
                "no-issues",
            );
            assert!(resumed.status.success(), "{resumed:?}");
            assert_eq!(test.calls().matches("worker [\"agent\"").count(), 1);
            assert!(!test.0.join(".git/autodev/loop.json").exists());
        }
    }
}

#[test]
fn loop_starts_next_issue_only_after_its_predecessor_pr_merges() {
    let test = Sandbox::new("loop-two");
    let result = test.invoke(
        &["loop", "--max-cycles", "2", "--poll-interval", "1"],
        "loop-two",
    );
    assert!(result.status.success(), "{result:?}");
    let calls = test.calls();
    assert_eq!(calls.matches("worker [\"agent\"").count(), 2);
    assert_eq!(calls.matches("gh [\"pr\", \"create\"").count(), 2);
    assert!(
        calls
            .find("\"view\", \"https://github.com/test/project/pull/1\"")
            .unwrap()
            < calls.rfind("worker [\"agent\"").unwrap()
    );
    assert!(test
        .0
        .join(".git/autodev/worktrees/issue-8/change.txt")
        .exists());
}

#[test]
fn publication_recovers_lost_response_and_notification_without_duplicate_pr() {
    for mode in ["pr-lost-response", "notify-fail"] {
        let test = Sandbox::new(mode);
        test.configure(|v| {
            v["workflow"] =
                serde_json::json!({"reviewers":["reviewer"],"notify":["worker","notify"]})
        });
        let result = test.invoke(&["run", "--issue", "7", "--publish"], mode);
        assert!(!result.status.success());
        let result = test.invoke(&["run", "--issue", "7", "--resume", "--publish"], "");
        assert!(result.status.success(), "{result:?}");
        let calls = test.calls();
        assert_eq!(calls.matches("gh [\"pr\", \"create\"").count(), 1);
        assert_eq!(calls.matches("worker [\"agent\"").count(), 1);
        assert!(calls.contains("\"--reviewer\", \"reviewer\""));
        assert!(calls.contains("worker [\"notify\", \"https://github.com/test/project/pull/1\"]"));
    }
}

#[test]
fn loop_resume_retries_incomplete_notification_without_republishing() {
    let test = Sandbox::new("loop-notification");
    test.configure(|v| v["workflow"] = serde_json::json!({"notify":["worker","notify"]}));
    let first = test.invoke(&["loop", "--max-cycles", "1"], "notify-fail");
    assert!(!first.status.success());
    let second = test.invoke(&["loop", "--max-cycles", "1"], "no-issues");
    assert!(second.status.success(), "{second:?}");
    let calls = test.calls();
    assert_eq!(calls.matches("worker [\"notify\"").count(), 2);
    assert_eq!(calls.matches("\"push\"").count(), 1);
    assert_eq!(calls.matches("gh [\"pr\", \"create\"").count(), 1);
}

#[test]
fn dry_run_reports_effective_publication_and_executes_nothing() {
    let test = Sandbox::new("policy-dry");
    test.configure(|v| v["workflow"] = serde_json::json!({"auto_push":true,"auto_pr":true}));
    let output = test.invoke(&["run", "--dry-run"], "");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("push: true; PR: true"));
    let output = test.invoke(&["run", "--dry-run", "--until", "validate"], "");
    assert!(String::from_utf8_lossy(&output.stdout).contains("push: false; PR: false"));
    assert!(test.invoke(&["loop", "--dry-run"], "").status.success());
    assert!(test.calls().is_empty());
    assert!(!test.0.join(".git").exists());
}

#[test]
fn project_checks_preserve_partial_staging_and_untracked_files() {
    let test = Sandbox::with_real_git("check-index");
    fs::write(test.0.join("partial.txt"), "staged version").unwrap();
    real_git(&test.0, &["add", "partial.txt"]);
    fs::write(test.0.join("partial.txt"), "unstaged version").unwrap();
    fs::write(test.0.join("untracked.txt"), "untracked").unwrap();
    let before = real_git(&test.0, &["diff", "--cached", "--binary"]);
    for mode in ["success", "check-fail"] {
        let result = test.invoke(&["project", "check"], mode);
        assert_eq!(result.status.success(), mode == "success", "{result:?}");
        assert_eq!(real_git(&test.0, &["diff", "--cached", "--binary"]), before);
        assert_eq!(
            fs::read_to_string(test.0.join("partial.txt")).unwrap(),
            "unstaged version"
        );
        assert!(real_git(&test.0, &["ls-files", "untracked.txt"]).is_empty());
    }
}

#[test]
fn init_specification_tools_and_project_result_gates() {
    let test = Sandbox::new("project-tools");
    assert!(test.invoke(&["init"], "").status.success());
    assert!(!test.0.join("workflow/roles").exists());
    let example: serde_json::Value =
        serde_json::from_str(include_str!("../autodev.example.json")).unwrap();
    for stage in example["workflow"]["stages"].as_array().unwrap() {
        assert!(test.0.join(stage["prompt"].as_str().unwrap()).is_file());
    }
    assert!(test.0.join("workflow/stages/implementation.md").is_file());
    assert!(test.0.join("workflow/templates/spec-template.md").is_file());
    assert!(test.calls().is_empty());
    fs::write(
        test.0.join("workflow/stages/implementation.md"),
        "customized",
    )
    .unwrap();
    assert!(test.invoke(&["init"], "").status.success());
    assert_eq!(
        fs::read_to_string(test.0.join("workflow/stages/implementation.md")).unwrap(),
        "customized"
    );
    fs::create_dir_all(test.0.join("specs/001-test")).unwrap();
    fs::write(test.0.join("specs/001-test/spec.md"), "Spec").unwrap();
    assert!(test
        .invoke(&["project", "plan", "specs/001-test"], "")
        .status
        .success());
    assert!(!test
        .invoke(&["project", "plan", "specs/001-test"], "")
        .status
        .success());
    assert!(test
        .invoke(&["project", "prerequisites", "specs/001-test"], "")
        .status
        .success());
    fs::write(test.0.join("AGENTS.md"), "Keep my instructions").unwrap();
    assert!(test
        .invoke(&["project", "context", "specs/001-test"], "")
        .status
        .success());
    assert!(fs::read_to_string(test.0.join("AGENTS.md"))
        .unwrap()
        .starts_with("Keep my instructions"));
    for mode in ["stage-blocked", "stage-missing", "success"] {
        let result = test.invoke(&["project", "prompt", "analyze", "specs/001-test"], mode);
        assert_eq!(result.status.success(), mode == "success", "{result:?}");
    }
}

#[test]
fn task_issue_tools_preserve_metadata_and_make_dry_run_local() {
    let test = Sandbox::new("task-issues");
    fs::write(test.0.join("tasks.md"), "## Phase 1: Setup (Priority: P1)\n**Goal**: Ready\n**Independent Test**: Builds\n- [ ] T001 [P] [US1] Implement API\n").unwrap();
    let result = test.invoke(&["project", "issues", "tasks.md", "--dry-run"], "");
    assert!(result.status.success(), "{result:?}");
    assert!(test.calls().is_empty());
    assert!(String::from_utf8_lossy(&result.stdout).contains("\"p1\""));
    let result = test.invoke(&["project", "issues", "tasks.md"], "");
    assert!(result.status.success(), "{result:?}");
    assert!(test.calls().contains("\"--label\", \"us1\""));
    assert!(test
        .invoke(&["project", "issue-labels"], "label-inference")
        .status
        .success());
    assert!(test.calls().contains("\"--add-label\", \"database\""));
    assert!(test.calls().contains("\"--add-label\", \"us2\""));
    assert!(test
        .invoke(&["project", "renumber", "tasks.md", "--offset", "2"], "")
        .status
        .success());
    assert!(fs::read_to_string(test.0.join("tasks.md"))
        .unwrap()
        .contains("T003"));
    assert!(fs::read_to_string(test.0.join("tasks.md.backup"))
        .unwrap()
        .contains("T001"));
    assert!(!test
        .invoke(&["project", "renumber", "tasks.md", "--offset", "2"], "")
        .status
        .success());
}

#[test]
fn feature_numbering_uses_local_and_remote_branches() {
    let test = Sandbox::with_real_git("feature-numbers");
    real_git(&test.0, &["branch", "005-existing"]);
    real_git(
        &test.0,
        &["push", "origin", "development:refs/heads/009-remote-only"],
    );
    let result = test.invoke(&["project", "feature", "new feature"], "");
    assert!(result.status.success(), "{result:?}");
    assert_eq!(
        real_git(&test.0, &["branch", "--show-current"]),
        "010-new-feature"
    );
    assert!(test.0.join("specs/010-new-feature/spec.md").exists());
}

#[test]
fn cleanup_keeps_original_checkout_and_refuses_untracked_work() {
    for dirty in [false, true] {
        let test = Sandbox::with_real_git(if dirty {
            "cleanup-dirty"
        } else {
            "cleanup-clean"
        });
        let result = test.invoke(&["run", "--issue", "7", "--publish"], "");
        assert!(result.status.success(), "{result:?}");
        let worktree = test.0.join(".git/autodev/worktrees/issue-7");
        if dirty {
            fs::write(worktree.join("valuable.txt"), "keep").unwrap();
        }
        let result = test.invoke(&["project", "cleanup", "--issue", "7"], "");
        assert_eq!(result.status.success(), !dirty, "{result:?}");
        assert_eq!(worktree.exists(), dirty);
        assert_eq!(
            real_git(&test.0, &["branch", "--show-current"]),
            "development"
        );
        assert!(test.0.join(".git/autodev/logs/issue-7/state.json").exists());
        assert!(!real_git(
            &test.0,
            &["ls-remote", "origin", "refs/heads/autodev/issue-7"]
        )
        .is_empty());
    }
}

#[cfg(unix)]
#[test]
fn cancellation_during_remote_check_prevents_push() {
    let test = Sandbox::new("cancel-before-push");
    let mut child = test
        .command(&["run", "--issue", "7", "--publish"], "cancel-before-push")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let start = std::time::Instant::now();
    while !test.0.join("cancel-ready").exists() {
        if start.elapsed() > std::time::Duration::from_secs(10) {
            let _ = child.kill();
            let _ = child.wait();
            panic!("remote check not reached");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(Command::new("kill")
        .args(["-INT", &child.id().to_string()])
        .status()
        .unwrap()
        .success());
    assert!(!child.wait().unwrap().success());
    assert!(!test.calls().contains("\"push\""));
}

fn real_git(root: &std::path::Path, args: &[&str]) -> String {
    let result = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    String::from_utf8(result.stdout).unwrap().trim().to_owned()
}

impl Sandbox {
    fn with_real_git(name: &str) -> Self {
        let mut test = Self::new(name);
        test.1 = Some(
            std::env::split_paths(&std::env::var_os("PATH").unwrap())
                .map(|path| path.join(format!("git{}", std::env::consts::EXE_SUFFIX)))
                .find(|path| path.is_file())
                .expect("Git executable on PATH"),
        );
        fs::write(
            test.0.join(".gitignore"),
            "bin/\ncalls\nchanged-remote\npr-created\nremote.git/\n",
        )
        .unwrap();
        real_git(&test.0, &["init", "-b", "development"]);
        real_git(&test.0, &["config", "user.name", "AutoDev test"]);
        real_git(&test.0, &["config", "user.email", "test@example.invalid"]);
        real_git(&test.0, &["add", "."]);
        real_git(&test.0, &["commit", "-m", "seed"]);
        real_git(&test.0, &["init", "--bare", "remote.git"]);
        real_git(
            &test.0,
            &[
                "remote",
                "add",
                "origin",
                &test.0.join("remote.git").to_string_lossy(),
            ],
        );
        real_git(&test.0, &["push", "origin", "development"]);
        test
    }
}

#[test]
fn real_git_cycle_publishes_only_the_checked_tree() {
    for mode in ["success", "check-fail", "hook-edits", "check-edits"] {
        let test = Sandbox::with_real_git(&format!("real-{mode}"));
        if mode == "hook-edits" {
            let hook = test.0.join(".git/hooks/pre-commit");
            fs::write(
                &hook,
                "#!/bin/sh\nprintf 'changed by hook' > change.txt\ngit add change.txt\n",
            )
            .unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
            }
        }
        let result = test.invoke(&["run", "--issue", "7", "--publish"], mode);
        assert_eq!(
            result.status.success(),
            mode == "success",
            "{mode}: {result:?}"
        );
        assert_eq!(
            real_git(&test.0, &["branch", "--show-current"]),
            "development"
        );
        assert!(real_git(&test.0, &["status", "--porcelain"]).is_empty());
        let remote_head = real_git(
            &test.0,
            &["ls-remote", "origin", "refs/heads/autodev/issue-7"],
        );
        assert_eq!(!remote_head.is_empty(), mode == "success");
        assert!(test
            .0
            .join(".git/autodev/worktrees/issue-7/change.txt")
            .exists());
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn backend_input_and_explicit_probe_are_provider_independent() {
    for input in ["argument", "stdin"] {
        for mode in ["success", "agent-fail", "timeout"] {
            let test = Sandbox::new(&format!("backend-{input}-{mode}"));
            let path = test.0.join("autodev.json");
            let mut config: serde_json::Value =
                serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            config["agent_input"] = input.into();
            if input == "stdin" {
                config["agent"] = serde_json::json!(["worker", "agent", "--stdin"]);
            }
            fs::write(&path, config.to_string()).unwrap();
            let doctor = test.invoke(&["doctor"], "");
            assert!(doctor.status.success(), "{doctor:?}");
            assert!(!test.calls().contains("worker"));
            config["agent_probe"] = serde_json::json!(["worker", "health"]);
            fs::write(&path, config.to_string()).unwrap();
            assert!(test.invoke(&["doctor"], "").status.success());
            assert!(test.calls().contains("worker [\"health\"]"));
            let result = test.invoke(&["run", "--issue", "7", "--publish"], mode);
            assert_eq!(
                result.status.success(),
                mode == "success",
                "{input}/{mode}: {result:?}"
            );
            assert_eq!(test.calls().contains("\"push\""), mode == "success");
        }
    }
}

#[test]
fn dry_run_has_no_processes_or_state_writes() {
    let test = Sandbox::new("dry");
    let result = test.invoke(&["run", "--issue", "7", "--dry-run", "--publish"], "");
    assert!(result.status.success(), "{:?}", result);
    assert!(test.calls().is_empty());
    assert!(!test.0.join(".git").exists());
}

#[test]
fn failure_and_timeout_preserve_work_and_never_publish() {
    for mode in ["agent-fail", "check-fail", "timeout"] {
        let test = Sandbox::new(mode);
        let result = test.invoke(&["run", "--issue", "7", "--publish"], mode);
        assert!(!result.status.success());
        assert!(
            test.0
                .join(".git/autodev/worktrees/issue-7/change.txt")
                .exists(),
            "{:?}",
            result
        );
        let calls = test.calls();
        assert!(!calls.contains("\"push\""));
        assert!(!calls.contains("\"commit\""));
        assert!(!test.0.join(".git/autodev/run.lock").exists());
        let status = test.invoke(&["status"], "");
        assert!(status.status.success());
        assert!(String::from_utf8_lossy(&status.stdout).contains("failed"));
    }
}

#[test]
fn validated_cycle_stops_locally_unless_publish_is_requested() {
    for publish in [false, true] {
        let test = Sandbox::new(if publish { "publish" } else { "local" });
        let mut args = vec!["run", "--issue", "7"];
        if publish {
            args.push("--publish");
        }
        let result = test.invoke(&args, "");
        assert!(result.status.success(), "{:?}", result);
        let calls = test.calls();
        assert!(calls.contains("\"commit\""));
        assert_eq!(calls.contains("\"push\""), publish);
        assert_eq!(calls.contains("\"--draft\""), publish);
        assert!(!calls.contains("\"merge\""));
        assert!(calls.find("worker [\"check\"]").unwrap() < calls.find("\"commit\"").unwrap());
        let second = test.invoke(&args, "");
        assert!(!second.status.success());
        assert!(String::from_utf8_lossy(&second.stderr).contains("existing work preserved"));
    }
}

#[test]
fn changed_remote_is_never_published() {
    let test = Sandbox::new("remote-change");
    let result = test.invoke(&["run", "--issue", "7", "--publish"], "remote-change");
    assert!(!result.status.success());
    assert!(!test.calls().contains("\"push\""));
    assert!(String::from_utf8_lossy(&result.stderr).contains("must match github_repo"));
}

#[test]
fn git_and_pr_errors_are_persisted_without_deleting_work() {
    for mode in ["commit-fail", "push-fail", "pr-fail"] {
        let test = Sandbox::new(mode);
        let result = test.invoke(&["run", "--issue", "7", "--publish"], mode);
        assert!(!result.status.success());
        assert!(test
            .0
            .join(".git/autodev/worktrees/issue-7/change.txt")
            .exists());
        let calls = test.calls();
        if mode == "commit-fail" {
            assert!(!calls.contains("\"push\""));
        }
        if mode != "pr-fail" {
            assert!(!calls.contains("gh [\"pr\""));
        }
        let status = test.invoke(&["status"], "");
        assert!(String::from_utf8_lossy(&status.stdout).contains("failed"));
    }
}
