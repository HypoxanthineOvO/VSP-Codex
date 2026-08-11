use std::path::Path;

use anyhow::Context;
use anyhow::Result;
use codex_login::REVOKE_TOKEN_URL_OVERRIDE_ENV_VAR;
use predicates::str::contains;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;
use tempfile::TempDir;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path;

fn codex_command(codex_home: &Path) -> Result<assert_cmd::Command> {
    let mut cmd = assert_cmd::Command::new(codex_utils_cargo_bin::cargo_bin("vsp-codex")?);
    cmd.env("CODEX_HOME", codex_home);
    Ok(cmd)
}

fn write_file_auth_config(codex_home: &Path) -> Result<()> {
    std::fs::write(
        codex_home.join("config.toml"),
        "cli_auth_credentials_store = \"file\"\n",
    )?;
    Ok(())
}

fn read_auth_json(codex_home: &Path) -> Result<Value> {
    let auth_path = codex_home.join("auth.json");
    let contents = std::fs::read_to_string(&auth_path)
        .with_context(|| format!("failed to read {}", auth_path.display()))?;
    Ok(serde_json::from_str(&contents)?)
}

#[test]
fn login_with_api_key_is_disabled_in_vsp() -> Result<()> {
    let codex_home = TempDir::new()?;
    write_file_auth_config(codex_home.path())?;

    let mut cmd = codex_command(codex_home.path())?;
    cmd.args([
        "-c",
        "forced_login_method=\"api\"",
        "login",
        "--with-api-key",
    ])
    .write_stdin("sk-test\n")
    .assert()
    .failure()
    .stderr(contains(
        "This official Codex service is disabled in VSP Codex.",
    ));

    Ok(())
}

#[test]
fn login_with_access_token_is_disabled_in_vsp() -> Result<()> {
    let codex_home = TempDir::new()?;
    write_file_auth_config(codex_home.path())?;

    let mut cmd = codex_command(codex_home.path())?;
    cmd.args(["login", "--with-access-token"])
        .write_stdin("not-a-jwt\n")
        .assert()
        .failure()
        .stderr(contains(
            "This official Codex service is disabled in VSP Codex.",
        ));

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn device_login_is_disabled_without_mutating_existing_auth() -> Result<()> {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth/revoke"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/accounts/deviceauth/usercode"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "device_auth_id": "device-auth-123",
            "user_code": "CODE-12345",
            "interval": "0",
        })))
        .expect(0)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/accounts/deviceauth/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "authorization_code": "authorization-code-123",
            "code_challenge": "code-challenge-123",
            "code_verifier": "code-verifier-123",
        })))
        .expect(0)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id_token": "eyJhbGciOiJub25lIn0.e30.c2ln",
            "access_token": "new-access",
            "refresh_token": "new-refresh",
        })))
        .expect(0)
        .mount(&server)
        .await;

    let codex_home = TempDir::new()?;
    write_file_auth_config(codex_home.path())?;
    std::fs::write(
        codex_home.path().join("auth.json"),
        serde_json::to_vec(&json!({
            "auth_mode": "chatgpt",
            "OPENAI_API_KEY": null,
            "tokens": {
                "id_token": "eyJhbGciOiJub25lIn0.e30.c2ln",
                "access_token": "old-access",
                "refresh_token": "old-refresh",
                "account_id": "old-account",
            },
        }))?,
    )?;

    let issuer = server.uri();
    let mut cmd = codex_command(codex_home.path())?;
    cmd.env(
        REVOKE_TOKEN_URL_OVERRIDE_ENV_VAR,
        format!("{issuer}/oauth/revoke"),
    )
    .env("NO_PROXY", "127.0.0.1,localhost")
    .env("no_proxy", "127.0.0.1,localhost")
    .env_remove("CODEX_ACCESS_TOKEN")
    .env_remove("OPENAI_API_KEY")
    .args(["login", "--device-auth", "--experimental_issuer", &issuer])
    .assert()
    .failure()
    .stderr(contains(
        "This official Codex service is disabled in VSP Codex.",
    ));

    let requests = server
        .received_requests()
        .await
        .context("failed to read mock OAuth requests")?;
    assert!(
        requests.is_empty(),
        "disabled VSP login must not call official OAuth endpoints: {requests:#?}"
    );

    let auth = read_auth_json(codex_home.path())?;
    assert_eq!(auth["tokens"]["refresh_token"], "old-refresh");
    Ok(())
}
