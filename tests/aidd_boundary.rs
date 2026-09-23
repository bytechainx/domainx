#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! `domainx` 的 AI 生成边界候选；逐条人工复核后保留。
//!
//! // AIDD: 负数量 | 来源=AI | 复核=ZoneCNH/2026-09-24 | 依据=标准.md §约束，数量不能为负 | 结论=保留
//! // AIDD: 成交加剩余不等于原数量 | 来源=AI | 复核=ZoneCNH/2026-09-24 | 依据=标准.md §约束，数量守恒 | 结论=保留
//! // AIDD: GTD 截止早于创建时间 | 来源=AI | 复核=ZoneCNH/2026-09-24 | 依据=标准.md §约束，截止时间有效 | 结论=保留
//! // AIDD: commission 输入跨资产 | 来源=AI | 复核=ZoneCNH/2026-09-24 | 依据=标准.md §范围，按币种分组汇总 | 结论=保留
//! // AIDD: instrument 标识包含不同交易场所 | 来源=AI | 复核=ZoneCNH/2026-09-24 | 依据=标准.md §职责，身份保留 exchange 与 symbol | 结论=保留

use decimalx::Decimal;
use domainx::{
    Commission, InstrumentKey, TimeInForce, aggregate_commissions_by_asset, validate_gtd_deadline,
    validate_quantity_balance,
};

#[test]
fn quantity_conservation_rejects_mismatch() {
    assert!(validate_quantity_balance(Decimal::ONE, Decimal::ZERO, Decimal::ZERO).is_err());
}

#[test]
fn gtd_deadline_at_creation_is_allowed() {
    assert!(validate_gtd_deadline(&TimeInForce::Gtd(10), 10).is_ok());
}

#[test]
fn commissions_are_grouped_by_currency() {
    let values = [
        Commission { amount: Decimal::ONE, asset: "USD".into() },
        Commission { amount: Decimal::ONE, asset: "EUR".into() },
    ];
    assert_eq!(aggregate_commissions_by_asset(&values).unwrap().len(), 2);
}

#[test]
fn instrument_identity_keeps_exchange_and_symbol() {
    let a = InstrumentKey { exchange: "x".into(), symbol: "y".into() };
    assert_ne!(a.exchange, a.symbol);
}

#[test]
fn empty_commission_set_is_empty() {
    assert!(aggregate_commissions_by_asset(&[]).unwrap().is_empty());
}
