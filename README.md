# domainx

`domainx` 提供共享交易领域值对象和纯函数校验，不执行交易或 I/O。

| 项目 | 值 |
| --- | --- |
| 版本 | `0.1.5` |
| Rust | Edition 2024，MSRV 1.88 |
| 许可 | MIT |
| 发布 | 仅从 Git 源码消费，不发布到 crates.io |

## 获取源码

`domainx` 依赖同级 `decimalx` 仓库，后者又依赖同级 `kernel` 仓库：

```bash
git clone git@github.com:bytechainx/kernel.git
git clone git@github.com:bytechainx/decimalx.git
git clone git@github.com:bytechainx/domainx.git
```

在 Cargo 项目中使用本地 checkout：

```toml
[dependencies]
domainx = { version = "0.1.5", path = "../domainx" }
```

## 职责与边界

- 定义订单、持仓、成交、执行回报、组合和共享枚举。
- `validate` 提供显式的纯函数校验。
- 不下单、不做风控、网络请求、持久化或 PnL 计算。
- 构造或反序列化不会自动拦截无效对象；需要时调用相应校验函数。

## 验证

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```
