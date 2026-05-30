use crate::order::{Order, OrderType, Side, Trade};
use crate::orderbook::OrderBook;
use crate::types::{OrderId, Price, Quantity};

#[derive(Debug)]
pub struct ExecutionReport {
    pub trades: Vec<Trade>,
    pub remaining_qty: Quantity,
    pub status: ExecStatus,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExecStatus {
    Filled,
    PartiallyFilled,
    Resting,
    Rejected(String),
}

pub struct MatchingEngine {
    book: OrderBook,
    _next_trade_seq: u64,
    trades_buf: Vec<Trade>,
}

impl MatchingEngine {
    pub fn new() -> Self {
        Self {
            book: OrderBook::new(Price(5_000_000)),
            _next_trade_seq: 0,
            trades_buf: Vec::with_capacity(64),
        }
    }

    pub fn book(&self) -> &OrderBook {
        &self.book
    }

    pub fn submit(&mut self, mut taker: Order) -> &[Trade] {
        self.trades_buf.clear();
        match taker.side {
            Side::Buy  => self.match_against_asks(&mut taker),
            Side::Sell => self.match_against_bids(&mut taker),
        }

        if taker.quantity.0 > 0 {
            match taker.order_type {
                OrderType::Limit => {
                    let _ = self.book.add(taker);
                }
                OrderType::Market => {}
            }
        }

        &self.trades_buf
    }

    fn match_against_asks(&mut self, taker: &mut Order) {
        while taker.quantity.0 > 0 {
            let best_price = match self.book.best_ask() {
                Some(p) => p,
                None => break,
            };
            if taker.order_type == OrderType::Limit && taker.price < best_price {
                break;
            }
            if !self.consume_level(Side::Sell, best_price, taker) {
                break;
            }
        }
    }

    fn match_against_bids(&mut self, taker: &mut Order) {
        while taker.quantity.0 > 0 {
            let best_price = match self.book.best_bid() {
                Some(p) => p,
                None => break,
            };
            if taker.order_type == OrderType::Limit && taker.price > best_price {
                break;
            }
            if !self.consume_level(Side::Buy, best_price, taker) {
                break;
            }
        }
    }

    fn consume_level(
        &mut self,
        maker_side: Side,
        price: Price,
        taker: &mut Order,
    ) -> bool {
        let idx = match self.book.price_to_idx(price) {
            Some(i) => i,
            None => return false,
        };

        let mut consumed: Vec<OrderId> = Vec::new();

        loop {
            if taker.quantity.0 == 0 { break; }

            // Read front order — immutable borrow ends before trades_buf access
            let front = self.book.levels[idx].as_ref()
                .and_then(|l| l.orders.front())
                .map(|m| (m.id, m.quantity.0));

            let (maker_id, maker_qty) = match front {
                Some(v) => v,
                None => break,
            };

            let exec_qty = taker.quantity.0.min(maker_qty);

            self.trades_buf.push(Trade {
                taker_order_id: taker.id,
                maker_order_id: maker_id,
                price,
                quantity: Quantity(exec_qty),
                timestamp: taker.timestamp,
            });

            taker.quantity.0 -= exec_qty;

            // Separate mutable borrow to update level
            let level = self.book.levels[idx].as_mut().unwrap();
            level.total_qty -= exec_qty;
            level.orders.front_mut().unwrap().quantity.0 -= exec_qty;
            if maker_qty == exec_qty {
                level.orders.pop_front();
                consumed.push(maker_id);
            }
        }

        for id in consumed {
            self.book.order_index.remove(&id);
        }

        let is_empty = self.book.levels[idx].as_ref().map_or(true, |l| l.orders.is_empty());
        if is_empty {
            self.book.levels[idx] = None;
            self.book.occupied[idx / 64] &= !(1u64 << (idx % 64));
            self.book.update_best_after_removal(maker_side, idx);
        }

        true
    }

    pub fn cancel(&mut self, order_id: OrderId) -> Option<Order> {
        self.book.cancel(order_id)
    }
}

impl Default for MatchingEngine {
    fn default() -> Self {
        Self::new()
    }
}
