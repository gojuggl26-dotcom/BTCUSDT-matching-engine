use std::collections::{BTreeMap, HashMap, VecDeque};
use crate::order::{Order, Side};
use crate::types::{OrderId, Price};

pub struct OrderBook {
    pub(crate) bids: BTreeMap<Price, VecDeque<Order>>,
    pub(crate) asks: BTreeMap<Price, VecDeque<Order>>,
    pub(crate) order_index: HashMap<OrderId, (Side, Price)>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            order_index: HashMap::new(),
        }
    }

    pub fn best_bid(&self) -> Option<Price> {
        self.bids.keys().next_back().copied()
    }

    pub fn best_ask(&self) -> Option<Price> {
        self.asks.keys().next().copied()
    }

    pub fn add(&mut self, order: Order) {
        let side = order.side;
        let price = order.price;
        let book = match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };
        book.entry(price)
            .or_insert_with(VecDeque::new)
            .push_back(order.clone());
        self.order_index.insert(order.id, (side, price));
    }

    pub fn cancel(&mut self, order_id: OrderId) -> Option<Order> {
        let (side, price) = self.order_index.remove(&order_id)?;
        let book = match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };
        let queue = book.get_mut(&price)?;
        let pos = queue.iter().position(|o| o.id == order_id)?;
        let removed = queue.remove(pos);
        if queue.is_empty() {
            book.remove(&price);
        }
        removed
    }
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}
