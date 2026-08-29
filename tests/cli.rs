use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn git(dir: &Path, args: &[&str]) {
    let status = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        status.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&status.stderr)
    );
}

fn write(dir: &Path, rel: &str, content: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn ripple(dir: &Path) -> Command {
    let mut command = Command::cargo_bin("ripple").unwrap();
    command.current_dir(dir);
    command
}

fn fixture() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    git(root, &["init", "-b", "main"]);
    git(root, &["config", "user.email", "test@test.invalid"]);
    git(root, &["config", "user.name", "test"]);
    write(
        root,
        "ripple.toml",
        r#"
[modules.core]
path = "libs/core"

[modules.api]
path = "services/api"
deps = ["core"]

[modules.web]
path = "apps/web"
deps = ["api"]

[modules.docs]
path = "docs"
"#,
    );
    write(root, "libs/core/lib.rs", "core\n");
    write(root, "services/api/main.rs", "api\n");
    write(root, "apps/web/app.ts", "web\n");
    write(root, "docs/readme.md", "docs\n");
    git(root, &["add", "-A"]);
    git(root, &["commit", "-m", "init"]);
    dir
}

fn on_feature_with_core_change(dir: &TempDir) {
    let root = dir.path();
    git(root, &["checkout", "-b", "feature"]);
    write(root, "libs/core/lib.rs", "core changed\n");
}

#[test]
fn changed_reports_direct_and_transitive_modules() {
    let dir = fixture();
    on_feature_with_core_change(&dir);
    ripple(dir.path())
        .arg("changed")
        .assert()
        .success()
        .stdout("api\ncore\nweb\n");
}

#[test]
fn changed_direct_only_excludes_dependents() {
    let dir = fixture();
    on_feature_with_core_change(&dir);
    ripple(dir.path())
        .args(["changed", "--direct-only"])
        .assert()
        .success()
        .stdout("core\n");
}

#[test]
fn changed_includes_untracked_files() {
    let dir = fixture();
    git(dir.path(), &["checkout", "-b", "feature"]);
    write(dir.path(), "docs/new.md", "new\n");
    ripple(dir.path())
        .arg("changed")
        .assert()
        .success()
        .stdout("docs\n");
}

#[test]
fn changed_staged_only_sees_the_index() {
    let dir = fixture();
    write(dir.path(), "docs/readme.md", "updated\n");
    git(dir.path(), &["add", "docs"]);
    write(dir.path(), "apps/web/app.ts", "unstaged\n");
    ripple(dir.path())
        .args(["changed", "--staged"])
        .assert()
        .success()
        .stdout("docs\n");
}

#[test]
fn changed_accepts_an_explicit_range() {
    let dir = fixture();
    on_feature_with_core_change(&dir);
    git(dir.path(), &["commit", "-am", "core change"]);
    write(dir.path(), "docs/readme.md", "worktree only\n");
    ripple(dir.path())
        .args(["changed", "main...HEAD"])
        .assert()
        .success()
        .stdout("api\ncore\nweb\n");
}

#[test]
fn changed_json_reports_status_via_and_files() {
    let dir = fixture();
    on_feature_with_core_change(&dir);
    let output = ripple(dir.path())
        .args(["changed", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let report: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let modules = report["modules"].as_array().unwrap();
    assert_eq!(modules.len(), 3);
    let core = modules.iter().find(|m| m["name"] == "core").unwrap();
    assert_eq!(core["status"], "direct");
    assert_eq!(core["files"][0], "libs/core/lib.rs");
    let web = modules.iter().find(|m| m["name"] == "web").unwrap();
    assert_eq!(web["status"], "indirect");
    assert_eq!(web["via"][0], "api");
    assert_eq!(web["via"][1], "core");
}

#[test]
fn changed_github_format_emits_a_matrix() {
    let dir = fixture();
    on_feature_with_core_change(&dir);
    ripple(dir.path())
        .args(["changed", "--format", "github"])
        .assert()
        .success()
        .stdout(
            "{\"include\":[{\"module\":\"api\"},{\"module\":\"core\"},{\"module\":\"web\"}]}\n",
        );
}

#[test]
fn changed_warns_on_unowned_files_and_strict_fails() {
    let dir = fixture();
    git(dir.path(), &["checkout", "-b", "feature"]);
    write(dir.path(), "stray.txt", "stray\n");
    ripple(dir.path())
        .arg("changed")
        .assert()
        .success()
        .stderr(predicate::str::contains("not owned by any module"));
    ripple(dir.path())
        .args(["changed", "--strict"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--strict"));
}

#[test]
fn changed_filter_restricts_output() {
    let dir = fixture();
    on_feature_with_core_change(&dir);
    ripple(dir.path())
        .args(["changed", "--filter", "web,docs"])
        .assert()
        .success()
        .stdout("web\n");
}

#[test]
fn changed_filter_is_repeatable() {
    let dir = fixture();
    on_feature_with_core_change(&dir);
    ripple(dir.path())
        .args(["changed", "--filter", "web", "--filter", "api"])
        .assert()
        .success()
        .stdout("api\nweb\n");
}

#[test]
fn changed_filter_rejects_unknown_modules() {
    let dir = fixture();
    on_feature_with_core_change(&dir);
    ripple(dir.path())
        .args(["changed", "--filter", "wbe"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("did you mean `web`?"));
}

#[test]
fn changed_filter_empty_result_exits_zero() {
    let dir = fixture();
    on_feature_with_core_change(&dir);
    ripple(dir.path())
        .args(["changed", "--filter", "docs"])
        .assert()
        .success()
        .stdout("");
}

#[test]
fn changed_filter_does_not_bypass_strict() {
    let dir = fixture();
    git(dir.path(), &["checkout", "-b", "feature"]);
    write(dir.path(), "stray.txt", "stray\n");
    ripple(dir.path())
        .args(["changed", "--strict", "--filter", "docs"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--strict"));
}

#[test]
fn changed_filter_applies_to_github_format() {
    let dir = fixture();
    on_feature_with_core_change(&dir);
    ripple(dir.path())
        .args(["changed", "--filter", "core", "--format", "github"])
        .assert()
        .success()
        .stdout("{\"include\":[{\"module\":\"core\"}]}\n");
}

#[test]
fn changed_with_no_changes_prints_nothing() {
    let dir = fixture();
    ripple(dir.path())
        .arg("changed")
        .assert()
        .success()
        .stdout("");
}

#[test]
fn changed_respects_configured_base_branch() {
    let dir = fixture();
    git(dir.path(), &["branch", "-m", "main", "trunk"]);
    let config = fs::read_to_string(dir.path().join("ripple.toml")).unwrap();
    write(
        dir.path(),
        "ripple.toml",
        &format!("base = \"trunk\"\n{config}"),
    );
    git(dir.path(), &["commit", "-am", "set base"]);
    git(dir.path(), &["checkout", "-b", "feature"]);
    write(dir.path(), "apps/web/app.ts", "changed\n");
    ripple(dir.path())
        .arg("changed")
        .assert()
        .success()
        .stdout("web\n");
}

#[test]
fn validate_accepts_a_good_config() {
    let dir = fixture();
    ripple(dir.path())
        .arg("validate")
        .assert()
        .success()
        .stdout(predicate::str::contains("4 modules"));
}

#[test]
fn validate_rejects_cycles() {
    let dir = fixture();
    write(
        dir.path(),
        "ripple.toml",
        r#"
[modules.a]
path = "libs/core"
deps = ["b"]

[modules.b]
path = "docs"
deps = ["a"]
"#,
    );
    ripple(dir.path())
        .arg("validate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("cycle"));
}

#[test]
fn validate_suggests_a_fix_for_unknown_deps() {
    let dir = fixture();
    write(
        dir.path(),
        "ripple.toml",
        r#"
[modules.core]
path = "libs/core"

[modules.api]
path = "services/api"
deps = ["cor"]
"#,
    );
    ripple(dir.path())
        .arg("validate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("did you mean `core`?"));
}

#[test]
fn validate_rejects_missing_paths() {
    let dir = fixture();
    write(
        dir.path(),
        "ripple.toml",
        r#"
[modules.ghost]
path = "does/not/exist"
"#,
    );
    ripple(dir.path())
        .arg("validate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn explain_shows_the_dependency_chain() {
    let dir = fixture();
    on_feature_with_core_change(&dir);
    ripple(dir.path())
        .args(["explain", "web"])
        .assert()
        .success()
        .stdout(predicate::str::contains("web -> api -> core"));
}

#[test]
fn explain_shows_matched_files_for_direct_changes() {
    let dir = fixture();
    on_feature_with_core_change(&dir);
    ripple(dir.path())
        .args(["explain", "core"])
        .assert()
        .success()
        .stdout(predicate::str::contains("libs/core/lib.rs"));
}

#[test]
fn explain_reports_unaffected_modules() {
    let dir = fixture();
    on_feature_with_core_change(&dir);
    ripple(dir.path())
        .args(["explain", "docs"])
        .assert()
        .success()
        .stdout(predicate::str::contains("not affected"));
}

#[test]
fn explain_rejects_unknown_modules_with_a_hint() {
    let dir = fixture();
    ripple(dir.path())
        .args(["explain", "cor"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("did you mean `core`?"));
}

#[test]
fn graph_lists_modules_and_walks_dependents() {
    let dir = fixture();
    ripple(dir.path())
        .arg("graph")
        .assert()
        .success()
        .stdout(predicate::str::contains("api -> core"));
    ripple(dir.path())
        .args(["graph", "core", "--dependents"])
        .assert()
        .success()
        .stdout(predicate::str::contains("api").and(predicate::str::contains("web")));
}

#[test]
fn graph_emits_dot_and_mermaid() {
    let dir = fixture();
    ripple(dir.path())
        .args(["graph", "--format", "dot"])
        .assert()
        .success()
        .stdout(predicate::str::contains("digraph ripple"))
        .stdout(predicate::str::contains("\"api\" -> \"core\";"));
    ripple(dir.path())
        .args(["graph", "--format", "mermaid"])
        .assert()
        .success()
        .stdout(predicate::str::contains("graph LR"))
        .stdout(predicate::str::contains("api --> core"));
}

#[test]
fn list_prints_all_modules() {
    let dir = fixture();
    ripple(dir.path())
        .arg("list")
        .assert()
        .success()
        .stdout("api\ncore\ndocs\nweb\n");
}

#[test]
fn list_json_emits_module_names() {
    let dir = fixture();
    let output = ripple(dir.path())
        .args(["list", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let names: Vec<String> = serde_json::from_slice(&output).unwrap();
    assert_eq!(names, ["api", "core", "docs", "web"]);
}

#[test]
fn init_scaffolds_and_refuses_overwrite() {
    let dir = TempDir::new().unwrap();
    ripple(dir.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("wrote"));
    assert!(dir.path().join("ripple.toml").exists());
    ripple(dir.path())
        .arg("init")
        .assert()
        .failure()
        .stderr(predicate::str::contains("refusing to overwrite"));
}

#[test]
fn changed_from_a_nested_directory_finds_the_root() {
    let dir = fixture();
    on_feature_with_core_change(&dir);
    ripple(&dir.path().join("apps/web"))
        .arg("changed")
        .assert()
        .success()
        .stdout("api\ncore\nweb\n");
}

#[test]
fn missing_config_gives_an_actionable_error() {
    let dir = TempDir::new().unwrap();
    ripple(dir.path())
        .arg("changed")
        .assert()
        .failure()
        .stderr(predicate::str::contains("ripple init"));
}
