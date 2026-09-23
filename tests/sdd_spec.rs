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
    let err = domainx::validate_quantity_balance(
        domainx::Decimal::new(2, 0),
        domainx::Decimal::new(1, 0),
        domainx::Decimal::ZERO,
    )
    .expect_err("成交量与剩余量之和必须等于订单量");
    assert!(matches!(err, domainx::ValidationError::Quantity(..)));
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
    let err = domainx::validate_non_negative_quantities(
        domainx::Decimal::new(-1, 0),
        domainx::Decimal::ZERO,
        domainx::Decimal::ZERO,
    )
    .expect_err("数量不得为负");
    assert!(matches!(err, domainx::ValidationError::Quantity(..)));
}
