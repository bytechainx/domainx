#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! T-PROPERTY：数量守恒、TIF serde 往返、同资产佣金结合律。
//!
//! 策略显式含非法三元组、GTD 早于 created、佣金近上界。禁止恒真断言。

use domainx::{
    Commission, Decimal, TimeInForce, Timestamp, ValidationError, aggregate_commissions_by_asset,
    validate_gtd_deadline, validate_non_negative_quantities, validate_quantity_balance,
};
use proptest::prelude::*;

fn arb_tif() -> impl Strategy<Value = TimeInForce> {
    prop_oneof![
        Just(TimeInForce::Gtc),
        Just(TimeInForce::Ioc),
        Just(TimeInForce::Fok),
        any::<i64>().prop_map(TimeInForce::Gtd),
    ]
}

/// 非负小额 + 近 `Decimal::MAX`，迫使溢出路径被抽到。
fn arb_commission_amount() -> impl Strategy<Value = Decimal> {
    prop_oneof![
        3 => (0i64..10_000).prop_map(|n| Decimal::new(i128::from(n), 0)),
        1 => Just(Decimal::MAX),
        1 => Just(Decimal::new(i128::MAX - 10, 0)),
    ]
}

proptest! {
    /// 合法三元组：filled + remaining == quantity 且均非负 → Ok。
    #[test]
    fn quantity_conservation_legal(filled in 0i64..10_000, remaining in 0i64..10_000) {
        let filled = Decimal::new(i128::from(filled), 0);
        let remaining = Decimal::new(i128::from(remaining), 0);
        let quantity = filled.checked_add(remaining).expect("小额之和可表示");
        prop_assert!(
            validate_non_negative_quantities(quantity, filled, remaining).is_ok(),
            "合法三元组不得被 DX-VAL-001 拒绝"
        );
        prop_assert!(
            validate_quantity_balance(quantity, filled, remaining).is_ok(),
            "合法三元组不得被 DX-VAL-002 拒绝"
        );
    }

    /// 非法三元组：quantity 比 filled+remaining 多出正增量 → Quantity。
    #[test]
    fn quantity_conservation_illegal_triple(
        filled in 0i64..10_000,
        remaining in 0i64..10_000,
        delta in 1i64..100,
    ) {
        let filled = Decimal::new(i128::from(filled), 0);
        let remaining = Decimal::new(i128::from(remaining), 0);
        let balanced = filled.checked_add(remaining).expect("小额之和可表示");
        let quantity = balanced.checked_add(Decimal::new(i128::from(delta), 0)).expect("增量可表示");
        let err = validate_quantity_balance(quantity, filled, remaining)
            .expect_err("破坏数量守恒必须失败");
        prop_assert!(
            matches!(err, ValidationError::Quantity(..)),
            "非法三元组须为 Quantity，得到 {err:?}"
        );
    }

    /// TIF adjacently tagged 往返；并断言 wire 含 `type`（Gtd 另有 `value`）。
    #[test]
    fn time_in_force_serde_roundtrip(tif in arb_tif()) {
        let value = serde_json::to_value(&tif).expect("serialize tif");
        prop_assert!(value.get("type").is_some(), "TIF wire 必须 tagged: {value}");
        match &tif {
            TimeInForce::Gtd(ts) => {
                prop_assert_eq!(value.get("value").and_then(serde_json::Value::as_i64), Some(*ts));
            }
            _ => {
                prop_assert!(value.get("value").is_none(), "无 content 变体不得带 value: {value}");
            }
        }
        let back: TimeInForce = serde_json::from_value(value).expect("deserialize tif");
        prop_assert_eq!(back, tif);
    }

    /// GTD 截止早于 created → Time（策略强制 early_by ≥ 1）。
    #[test]
    fn gtd_before_created_is_time_error(created in 1_000i64..2_000_000_000, early_by in 1i64..50_000) {
        let created: Timestamp = created;
        let tif = TimeInForce::Gtd(created - early_by);
        let err = validate_gtd_deadline(&tif, created).expect_err("GTD 早于 created 必须失败");
        prop_assert!(matches!(err, ValidationError::Time(_)), "须为 Time，得到 {err:?}");
    }

    /// 同资产三笔结合：(a⊕b)⊕c 与 a⊕(b⊕c)；溢出两侧同为 Quantity。
    #[test]
    fn aggregate_same_asset_associative(
        a in arb_commission_amount(),
        b in arb_commission_amount(),
        c in arb_commission_amount(),
    ) {
        let ca = Commission { amount: a, asset: "USDT".into() };
        let cb = Commission { amount: b, asset: "USDT".into() };
        let cc = Commission { amount: c, asset: "USDT".into() };

        let left = aggregate_commissions_by_asset(&[ca.clone(), cb.clone()]).and_then(|ab| {
            let mut items = ab;
            items.push(cc.clone());
            aggregate_commissions_by_asset(&items)
        });
        let right = aggregate_commissions_by_asset(&[cb, cc]).and_then(|bc| {
            let mut items = vec![ca];
            items.extend(bc);
            aggregate_commissions_by_asset(&items)
        });

        match (left, right) {
            (Ok(l), Ok(r)) => prop_assert_eq!(l, r),
            (Err(el), Err(er)) => {
                prop_assert!(
                    matches!(el, ValidationError::Quantity(..)),
                    "左溢出须为 Quantity，得到 {el:?}"
                );
                prop_assert!(
                    matches!(er, ValidationError::Quantity(..)),
                    "右溢出须为 Quantity，得到 {er:?}"
                );
            }
            (Ok(l), Err(er)) => {
                return Err(TestCaseError::fail(format!(
                    "结合律两侧不一致: 左 Ok({l:?}) 右 Err({er:?})"
                )));
            }
            (Err(el), Ok(r)) => {
                return Err(TestCaseError::fail(format!(
                    "结合律两侧不一致: 左 Err({el:?}) 右 Ok({r:?})"
                )));
            }
        }
    }
}
