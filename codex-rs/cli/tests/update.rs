use anyhow::Result;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
#[cfg(not(debug_assertions))]
use std::fs;
#[cfg(not(debug_assertions))]
use std::io::Read;
#[cfg(not(debug_assertions))]
use std::io::Write;
#[cfg(not(debug_assertions))]
use std::net::TcpListener;
#[cfg(debug_assertions)]
use std::path::Path;
#[cfg(not(debug_assertions))]
use std::thread;
#[cfg(not(debug_assertions))]
use std::time::Duration;
#[cfg(not(debug_assertions))]
use std::time::Instant;
use tempfile::TempDir;

#[cfg(debug_assertions)]
fn codex_command(codex_home: &Path) -> Result<assert_cmd::Command> {
    let mut cmd = assert_cmd::Command::new(codex_utils_cargo_bin::cargo_bin("vsp-codex")?);
    cmd.env("CODEX_HOME", codex_home);
    Ok(cmd)
}

#[cfg(debug_assertions)]
#[tokio::test]
async fn update_does_not_start_interactive_prompt() -> Result<()> {
    let codex_home = TempDir::new()?;

    codex_command(codex_home.path())?
        .arg("update")
        .assert()
        .failure()
        .stderr(contains("not available in debug builds"))
        .stderr(contains("official Codex service is disabled").not());

    Ok(())
}

#[cfg(not(debug_assertions))]
#[test]
fn plain_update_discovers_vsp_manifest_from_install_context() -> Result<()> {
    let codex_home = TempDir::new()?;
    let target = "x86_64-unknown-linux-gnu";
    let release_dir = codex_home
        .path()
        .join("packages/standalone/releases")
        .join(format!("{}-{target}", env!("CARGO_PKG_VERSION")));
    let bin_dir = release_dir.join("bin");
    fs::create_dir_all(&bin_dir)?;

    let binary_name = if cfg!(windows) {
        "vsp-codex.exe"
    } else {
        "vsp-codex"
    };
    let installed_binary = bin_dir.join(binary_name);
    fs::copy(
        codex_utils_cargo_bin::cargo_bin("vsp-codex")?,
        &installed_binary,
    )?;

    let manifest = serde_json::json!({
        "schema": 1,
        "minimum_updater": "0.145.0-vsp.9",
        "releases": [{
            "version": env!("CARGO_PKG_VERSION"),
            "channel": "lab",
            "target": target,
            "archive_url": "https://releases.vsp.example/current.tar.gz",
            "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "notes_url": "https://releases.vsp.example/current"
        }]
    })
    .to_string();
    let (manifest_url, server) = serve_manifest_once(manifest)?;
    fs::write(
        release_dir.join("codex-package.json"),
        serde_json::json!({
            "vspRelease": {
                "manifestUrl": manifest_url,
                "channel": "lab",
                "source": "preview-release-builder",
                "target": target
            }
        })
        .to_string(),
    )?;

    let assert = assert_cmd::Command::new(&installed_binary)
        .env("CODEX_HOME", codex_home.path())
        .arg("update")
        .assert()
        .failure();
    server
        .join()
        .map_err(|_| anyhow::anyhow!("manifest server thread panicked"))??;
    assert
        .stderr(contains("No newer VSP release matched this install."))
        .stderr(contains("official Codex service is disabled").not());

    Ok(())
}

#[cfg(not(debug_assertions))]
fn serve_manifest_once(body: String) -> Result<(String, thread::JoinHandle<Result<()>>)> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    listener.set_nonblocking(true)?;
    let address = listener.local_addr()?;
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        anyhow::bail!("timed out waiting for manifest request");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => return Err(err.into()),
            }
        };
        let mut request = [0_u8; 4096];
        let _ = stream.read(&mut request)?;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes())?;
        Ok(())
    });

    Ok((format!("http://{address}/manifest.json"), handle))
}
