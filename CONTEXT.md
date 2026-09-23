# 项目上下文

`domainx` 是纯交易领域值对象 crate，当前版本 `0.1.5`。

- 语义权威：[`docs/标准.md`](docs/标准.md)。
- `decimalx` 是唯一内部 crate 依赖，必须保持 `version + path` 与版本一致。
- 领域校验是纯函数；crate 不承担交易执行、风控、网络或持久化。
- Rust Edition 2024，MSRV 1.88；通过 Git checkout 消费，不发布到 crates.io。
