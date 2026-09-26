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
            "bin/\ncalls\nchanged-remote\nremote.git/\n",
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
