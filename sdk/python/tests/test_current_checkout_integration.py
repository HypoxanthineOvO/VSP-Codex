import os
from pathlib import Path

import pytest

from openai_codex import Codex, CodexConfig

CODEX_PATH = os.environ.get("CODEX_PATH")


@pytest.mark.skipif(
    not CODEX_PATH,
    reason="run through the sdk-python test group with the current checkout binary",
)
def test_current_checkout_initializes_and_lists_models(tmp_path: Path) -> None:
    codex_home = tmp_path / "codex-home"
    codex_home.mkdir()
    env = os.environ.copy()
    env["CODEX_HOME"] = str(codex_home)

    with Codex(CodexConfig(cwd=str(tmp_path), env=env)) as codex:
        models = codex.models(include_hidden=True)
        metadata = codex.metadata

    assert metadata.userAgent is not None
    assert metadata.userAgent.strip()
    assert metadata.serverInfo is not None
    assert metadata.serverInfo.name is not None
    assert metadata.serverInfo.name.strip()
    assert models.data
