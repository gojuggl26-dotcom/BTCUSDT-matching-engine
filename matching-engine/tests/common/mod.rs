//! 統合テスト用ヘルパー

use matching_engine::{Order, OrderId, OrderType, Price, Quantity, Side, Timestamp};

pub fn limit_order(
    id: u64,
    side: Side,
    price: i64,
    qty: u64,
    ts: u64,
) -> Order {
    Order {
        id: OrderId(id),
        side,
        order_type: OrderType::Limit,
        price: Price(price),
        quantity: Quantity(qty),
        timestamp: Timestamp(ts),
    }
}

pub fn market_order(id: u64, side: Side, qty: u64, ts: u64) -> Order {
    Order {
        id: OrderId(id),
        side,
        order_type: OrderType::Market,
        price: Price(0),
        quantity: Quantity(qty),
        timestamp: Timestamp(ts),
    }
}
