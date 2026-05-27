use crate::types::{OrderId, Price, Quantity, Timestamp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn opposite(&self) -> Side {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]

pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone, PartialEq, Eq)]

pub struct Order {
    pub id: OrderId,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Price,
    pub quantity: Quantity,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone)]

pub struct Trade {
    pub taker_order_id: OrderId,
    pub maker_order_id: OrderId,
    pub price: Price,             // 約定価格 = makerの指値
    pub quantity: Quantity,
    pub timestamp: Timestamp,
}

