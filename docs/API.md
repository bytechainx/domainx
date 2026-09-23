# domainx 公开 API

`domainx` 定义共享订单、持仓、成交、资产组合和值对象校验，不连接交易场所，也不执行写入。

## 身份与基础类型

- `InstrumentKey`：以 `exchange` 与 `symbol` 构成场所内标的身份。
- `OrderId`、`TradeId`、`ReportId`、`PositionId`、`PortfolioId`：跨层共享标识。
- `OrderSide`、`OrderType`、`OrderStatus`、`TimeInForce`：订单字段和值域。
- `PositionDirection`、`PositionStatus`、`ExecType`：持仓与执行状态。

## 交易对象

- `Order`：订单及数量、价格、有效期和时间戳。
- `Position`：持仓方向、数量与入场价格。
- `Trade`、`ExecutionReport`：成交事实及执行回报。
- `Portfolio`：资产组合值对象。
- `Commission`：金额与资产标识。

## 校验与汇总

- `validate_order`、`validate_order_prices`：校验订单字段之间的约束。
- `validate_non_negative_quantities`、`validate_quantity_balance`：校验数量非负与守恒。
- `validate_created_before_updated`、`validate_gtd_deadline`：校验时间顺序。
- `aggregate_commissions_by_asset`：以 checked arithmetic 按资产汇总手续费。
