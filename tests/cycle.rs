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

struct Sandbox(PathBuf);
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
        Self(dir)
    }
    fn invoke(&self, args: &[&str], mode: &str) -> Output {
        let mut paths = vec![self.0.join("bin")];
        paths.extend(std::env::split_paths(
            &std::env::var_os("PATH").unwrap_or_default(),
        ));
        Command::new(env!("CARGO_BIN_EXE_autodev"))
            .args(["--repo"])
            .arg(&self.0)
            .args(args)
            .env("PATH", std::env::join_paths(paths).unwrap())
            .env("AUTODEV_TEST_DIR", &self.0)
            .env("AUTODEV_TEST_MODE", mode)
            .output()
            .unwrap()
    }
    fn calls(&self) -> String {
        fs::read_to_string(self.0.join("calls")).unwrap_or_default()
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
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
