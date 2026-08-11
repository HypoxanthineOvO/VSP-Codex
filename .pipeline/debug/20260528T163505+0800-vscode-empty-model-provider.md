# VSCode Plugin Resume Empty Model Provider

- 时间：2026-05-28 16:35:05 +08:00
- 症状：打开 VSP Codex VSCode 插件时报错 `Failed to resume chat`，底层错误为 `failed to load configuration: Model provider `` not found`。
- 影响：插件恢复旧 chat/session 时无法进入会话；普通 TUI 启动不一定触发，取决于传入的 provider override 或历史 metadata。

## 根因

`codex-rs/app-server` 在恢复会话时会把持久化的 `ThreadMetadata.model_provider` 写入 `ConfigOverrides.model_provider`。旧会话或迁移数据可能保存了空字符串 provider。随后 `codex-rs/core` 解析配置时把空字符串当作真实 provider id 查找，最终报出 `Model provider `` not found`。

## 修复

- `codex-rs/app-server/src/request_processors/thread_processor.rs`：恢复会话合并历史 metadata 时，只有非空 `model_provider` 才写入 typesafe override。
- `codex-rs/core/src/config/mod.rs`：配置加载时把空白 `model_provider` override 和空白 config provider 都视为未设置，继续按正常优先级回退到配置值或默认 `openai`。

## 测试

- `just fmt`
- `just test -p codex-core load_config_ignores_empty_model_provider_override`
- `just test -p codex-core load_config_uses_default_model_provider_when_configured_provider_is_empty`
- `just test -p codex-app-server merge_persisted_resume_metadata_ignores_empty_model_provider`
- `just fix -p codex-core`
- `just fix -p codex-app-server`
- `git diff --check`

## 预期行为

安装包含此修复的版本后，VSCode 插件恢复 provider 为空的旧会话时不再失败；如果没有显式 provider，会继续使用有效配置或默认 `openai`。

## 风险和后续

- 这是兼容性修复，不改变非空 provider 的行为。
- 需要发一个补丁预览版，让插件用户安装新包后才能拿到修复。
