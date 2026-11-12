# 变更记录

本仓库从独立 Git 源码版本 `0.1.5` 开始维护，不发布到 crates.io。

## [Unreleased]

## [0.1.6] - 2026-11-12

### 变更

- 将本体角色登记为 `kernel`，与 Feature 008 冻结名册一致；公开 API 不变。
- `decimalx` 由 `path = "../decimalx"` 改为 Git 依赖（Goal 029 T010 / DDR-028）；只克隆本仓即可构建。

### 说明

- 0.1.6 按 GOAL-DOMAINX-016 作为内部制品发布；不发布到 crates.io。
- 公开 API 与 0.1.5 相同。

## [0.1.5] - 2026-09-23 — bytechainx 初始迁移

### 说明

- 从 `xhyper.rs` 迁移领域值对象、纯函数校验、测试、示例和基准代码，保留 crate 版本、API 与序列化结构。
- 将 workspace 外部的 InstrumentKey 描述改为 crate 自包含语义。
