# v0.145.0-vsp.11.0-alpha.1

VSP 11.0 从 9.5 重新开发，继续以 OpenAI Responses 兼容接口为唯一 Provider 主线，不包含已废弃 10.x 线的 Chat API 实现。本版本先以 `alpha` 频道发布；默认 `lab` 频道仍保持在 9.5。

## 主要变化

- 多终端接入：同一会话的请求弹窗会向所有有效终端同步，首个有效回答生效；断线、恢复、取消订阅和完成状态会一致收敛。
- 思维链翻译：重写流式翻译归并，修复星号、Markdown emphasis、UTF-8 分块、修订替换、重复片段和窄终端重排问题。
- 日志与状态库：默认持久化过滤 TRACE/INFO 噪声，升级时安全轮换旧 `logs_2`；`state_5` 保留并收敛写频率、锁竞争和兼容读取。
- 性能与内存：减少 metadata 重复写入和兼容读取开销，加入多进程共享 HOME 压力、SQLite 完整性、RSS 与启动耗时证据。
- CI：改为长期稳定的最小合并契约，只检查可构建、CLI 基本交互、TUI 可见输出和 app-server 基本往返，不再硬编码版本、测试数量或快照数量。
- 发布链路：updater 和安装器支持 `0.145.0-vsp.11.0-alpha.1`，并将 alpha 与稳定 `lab` 频道隔离。

## 安装

显式选择 alpha 频道：

```shell
curl -fsSL https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/scripts/install/install.sh | VSP_CODEX_RELEASE_CHANNEL=alpha sh
```

安装后验证：

```shell
vsp-codex --version
codex --version
```

两个入口都应输出 `vsp-codex 0.145.0-vsp.11.0-alpha.1`。

## 验证

- Rust updater 测试：23 项通过。
- 发布脚本与安装器测试：8 项通过。
- 最小运行 smoke：源码 release 二进制与解包二进制均通过 7 项检查。
- 隔离安装：归档安装、两个命令入口和 current symlink 全部通过。
- 归档 SHA-256：`a09fcb11b29704528ab8fbd1cc124957280e95b4708d2a9f31867f55f75a238a`。
