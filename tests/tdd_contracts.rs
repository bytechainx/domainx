#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! `domainx` 的公开行为入口探针。
//!
//! // TDD-PROBE: InstrumentKey | 变异：丢弃 exchange 身份 | 红=instrument_key_retains_both_parts | 绿=instrument_key_retains_both_parts
//! // TDD-PROBE: Order | 变异：序列化时丢弃订单标识 | 红=order_round_trips_shared_fields | 绿=order_round_trips_shared_fields
//! // TDD-PROBE: OrderStatus | 变异：把 Filled 编码为 new | 红=order_status_uses_stable_wire_name | 绿=order_status_uses_stable_wire_name
//! // TDD-PROBE: validate_non_negative_quantities | 变异：接受负 quantity | 红=negative_order_quantity_is_rejected | 绿=negative_order_quantity_is_rejected
//! // TDD-PROBE: validate_quantity_balance | 变异：不比较 filled 与 remaining 总量 | 红=quantity_balance_is_enforced | 绿=quantity_balance_is_enforced
//! // TDD-PROBE: validate_gtd_deadline | 变异：接受早于创建时间的截止值 | 红=gtd_deadline_cannot_precede_creation | 绿=gtd_deadline_cannot_precede_creation
//! // TDD-PROBE: aggregate_commissions_by_asset | 变异：跨资产合并手续费 | 红=commission_totals_keep_asset_identity | 绿=commission_totals_keep_asset_identity

use decimalx::Decimal;
use domainx::{
    Commission, InstrumentKey, Order, OrderSide, OrderStatus, OrderType, TimeInForce,
    aggregate_commissions_by_asset, validate_gtd_deadline, validate_non_negative_quantities,
    validate_quantity_balance,
};

#[test]
fn instrument_key_retains_both_parts() {
    let key = InstrumentKey { exchange: "binance".into(), symbol: "BTCUSDT".into() };
    assert_eq!(key.exchange, "binance");
    assert_eq!(key.symbol, "BTCUSDT");
}

#[test]
fn order_round_trips_shared_fields() {
    let order = Order {
        order_id: "o-1".into(),
        instrument: InstrumentKey { exchange: "binance".into(), symbol: "BTCUSDT".into() },
        side: OrderSide::Buy,
        order_type: OrderType::Market,
        status: OrderStatus::New,
        price: None,
        stop_price: None,
        quantity: Decimal::ONE,
        filled_quantity: Decimal::ZERO,
        remaining_quantity: Decimal::ONE,
        avg_fill_price: None,
        time_in_force: TimeInForce::Gtc,
        created_at: 10,
        updated_at: 10,
        client_order_id: None,
    };
    let encoded = serde_json::to_string(&order).unwrap();
    let decoded: Order = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded, order);
}

#[test]
fn order_status_uses_stable_wire_name() {
    assert!(matches!(serde_json::to_string(&OrderStatus::Filled).as_deref(), Ok("\"filled\"")));
}

#[test]
fn negative_order_quantity_is_rejected() {
    assert!(
        validate_non_negative_quantities(Decimal::new(-1, 0), Decimal::ZERO, Decimal::ZERO)
            .is_err()
    );
}

#[test]
fn quantity_balance_is_enforced() {
    assert!(validate_quantity_balance(Decimal::ONE, Decimal::ZERO, Decimal::ZERO).is_err());
    assert!(validate_quantity_balance(Decimal::ONE, Decimal::ZERO, Decimal::ONE).is_ok());
}

#[test]
fn gtd_deadline_cannot_precede_creation() {
    assert!(validate_gtd_deadline(&TimeInForce::Gtd(9), 10).is_err());
    assert!(validate_gtd_deadline(&TimeInForce::Gtd(10), 10).is_ok());
}

#[test]
fn commission_totals_keep_asset_identity() {
    let values = [
        Commission { amount: Decimal::ONE, asset: "USD".into() },
        Commission { amount: Decimal::ONE, asset: "EUR".into() },
        Commission { amount: Decimal::ONE, asset: "USD".into() },
    ];
    let totals = aggregate_commissions_by_asset(&values);
    assert!(totals.is_ok());
    let totals = totals.unwrap();
    assert_eq!(totals.len(), 2);
    assert_eq!(
        totals.iter().find(|x| x.asset == "USD").map(|x| x.amount),
        Some(Decimal::new(2, 0))
    );
}
