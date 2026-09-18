#![cfg(unix)]

use std::{fs, os::unix::fs::PermissionsExt, process::Command};

use git2::{Oid, Repository, Signature};
use jj_spr::jj::Jujutsu;
use tempfile::{TempDir, tempdir};

struct Fixture {
    _dir: TempDir,
    jj: Jujutsu,
    remote: Repository,
    pushed: Oid,
    head: Oid,
    hook_input: std::path::PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let dir = tempdir().unwrap();
        let repo = Repository::init(dir.path().join("local")).unwrap();
        let remote = Repository::init_bare(dir.path().join("remote.git")).unwrap();
        let signature = Signature::now("Test User", "test@example.com").unwrap();
        let tree_id = repo.treebuilder(None).unwrap().write().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let pushed = repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Selected revision",
                &tree,
                &[],
            )
            .unwrap();
        let parent = repo.find_commit(pushed).unwrap();
        let head = repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Working copy",
                &tree,
                &[&parent],
            )
            .unwrap();
        let output = Command::new("jj")
            .args(["git", "init", "--colocate"])
            .current_dir(repo.workdir().unwrap())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let jj = Jujutsu::new(repo.workdir().unwrap().to_path_buf()).unwrap();
        assert_eq!(jj.git_repo.head().unwrap().target(), Some(head));

        let hooks = dir.path().join("hooks");
        fs::create_dir(&hooks).unwrap();
        let hook = hooks.join("pre-push");
        fs::write(&hook, "#!/bin/sh\ncat > \"$HOOK_INPUT\"\nexit 1\n").unwrap();
        fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
        repo.config()
            .unwrap()
            .set_str("core.hooksPath", hooks.to_str().unwrap())
            .unwrap();
        let hook_input = dir.path().join("hook-input");
        Self {
            _dir: dir,
            jj,
            remote,
            pushed,
            head,
            hook_input,
        }
    }

    async fn push(&self, no_verify: bool, refspec: &str) -> std::process::Output {
        self.jj
            .git_push_command(no_verify)
            .current_dir(self.jj.git_repo.workdir().unwrap())
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("HOOK_INPUT", &self.hook_input)
            .arg("--")
            .arg(self.remote.path())
            .arg(refspec)
            .output()
            .await
            .unwrap()
    }

    fn remote_head(&self) -> Option<Oid> {
        self.remote
            .find_reference("refs/heads/review")
            .ok()
            .and_then(|r| r.target())
    }
}

#[tokio::test]
async fn hook_rejection_blocks_creation_and_updates_unless_explicitly_skipped() {
    let fixture = Fixture::new();
    let refspec = format!("{}:refs/heads/review", fixture.pushed);
    assert!(!fixture.push(false, &refspec).await.status.success());
    assert_eq!(fixture.remote_head(), None);
    let input = fs::read_to_string(&fixture.hook_input).unwrap();
    let fields: Vec<_> = input.split_whitespace().collect();
    assert_eq!(
        fields,
        [
            fixture.pushed.to_string(),
            fixture.pushed.to_string(),
            "refs/heads/review".to_string(),
            Oid::zero().to_string()
        ]
    );
    assert_ne!(fixture.pushed, fixture.head);

    fs::remove_file(&fixture.hook_input).unwrap();
    assert!(fixture.push(true, &refspec).await.status.success());
    assert_eq!(fixture.remote_head(), Some(fixture.pushed));
    assert!(!fixture.hook_input.exists());

    let update = format!("{}:refs/heads/review", fixture.head);
    assert!(!fixture.push(false, &update).await.status.success());
    assert_eq!(fixture.remote_head(), Some(fixture.pushed));
    assert!(fixture.push(true, &update).await.status.success());
    assert_eq!(fixture.remote_head(), Some(fixture.head));
}

#[tokio::test]
async fn hook_rejection_blocks_deletion_unless_explicitly_skipped() {
    let fixture = Fixture::new();
    let refspec = format!("{}:refs/heads/review", fixture.pushed);
    assert!(fixture.push(true, &refspec).await.status.success());
    assert!(
        !fixture
            .push(false, ":refs/heads/review")
            .await
            .status
            .success()
    );
    assert_eq!(fixture.remote_head(), Some(fixture.pushed));
    let input = fs::read_to_string(&fixture.hook_input).unwrap();
    let fields: Vec<_> = input.split_whitespace().collect();
    assert_eq!(
        fields,
        [
            "(delete)".to_string(),
            Oid::zero().to_string(),
            "refs/heads/review".to_string(),
            fixture.pushed.to_string()
        ]
    );
    fs::remove_file(&fixture.hook_input).unwrap();
    assert!(
        fixture
            .push(true, ":refs/heads/review")
            .await
            .status
            .success()
    );
    assert_eq!(fixture.remote_head(), None);
    assert!(!fixture.hook_input.exists());
}

#[test]
fn push_commands_expose_explicit_hook_opt_out() {
    for command in ["diff", "land", "close", "cleanup"] {
        let output = Command::new(env!("CARGO_BIN_EXE_jj-spr"))
            .args([command, "--no-verify", "--help"])
            .output()
            .unwrap();
        assert!(output.status.success());
        let help = String::from_utf8(output.stdout).unwrap();
        assert!(help.contains("--no-verify"));
        assert!(help.contains("hooks run by default"));
    }
}
