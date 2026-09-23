#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! `domainx` 标准章节对应的可执行规格检查。
//!
//! // SPEC-MAP: X-1 | 职责 | assert_duties
//! // SPEC-MAP: X-2 | 范围 | assert_scope
//! // SPEC-MAP: X-3 | 约束 | assert_constraints

const STANDARD: &str = include_str!("../docs/标准.md");

#[test]
fn assert_duties() {
    assert!(STANDARD.contains("## 职责"));
    assert!(STANDARD.contains("值对象"));
}

#[test]
fn assert_scope() {
    assert!(STANDARD.contains("## 范围"));
    assert!(STANDARD.contains("InstrumentKey"));
}

#[test]
fn assert_constraints() {
    assert!(STANDARD.contains("## 约束"));
    assert!(STANDARD.contains("显式调用校验函数"));
}
