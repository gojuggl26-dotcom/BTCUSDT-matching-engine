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
    next_trade_seq: u64,
}

impl MatchingEngine {
    pub fn new() -> Self {
        Self {
            book: OrderBook::new(),
            next_trade_seq: 0,
        }
    }

    pub fn book(&self) -> &OrderBook {
        &self.book
    }

    pub fn submit(&mut self, mut taker: Order) -> ExecutionReport {
        let mut trades = Vec::new();
        match taker.side {
            Side::Buy => self.match_against_asks(&mut taker, &mut trades),
            Side::Sell => self.match_against_bids(&mut taker, &mut trades),
        }

        let status = if taker.quantity.0 == 0 {
            ExecStatus::Filled
        } else {
            match taker.order_type {
                OrderType::Limit => {
                    let was_partial = !trades.is_empty();
                    self.book.add(taker.clone());
                    if was_partial {
                        ExecStatus::PartiallyFilled
                    } else {
                        ExecStatus::Resting
                    }
                }
                OrderType::Market => {
                    if trades.is_empty() {
                        ExecStatus::Rejected("no liquidity".into())
                    } else {
                        ExecStatus::PartiallyFilled
                    }
                }
            }
        };

        ExecutionReport {
            trades,
            remaining_qty: taker.quantity,
            status,
        }
    }

    fn match_against_asks(&mut self, taker: &mut Order, trades: &mut Vec<Trade>) {
        while taker.quantity.0 > 0 {
            let best_price = match self.book.asks.keys().next().copied() {
                Some(p) => p,
                None => break,
            };
            if taker.order_type == OrderType::Limit && taker.price < best_price {
                break;
            }
            if !self.consume_level(Side::Sell, best_price, taker, trades) {
                break;
            }
        }
    }

    fn match_against_bids(&mut self, taker: &mut Order, trades: &mut Vec<Trade>) {
        while taker.quantity.0 > 0 {
            let best_price = match self.book.bids.keys().next_back().copied() {
                Some(p) => p,
                None => break,
            };
            if taker.order_type == OrderType::Limit && taker.price > best_price {
                break;
            }
            if !self.consume_level(Side::Buy, best_price, taker, trades) {
                break;
            }
        }
    }

    fn consume_level(
        &mut self,
        maker_side: Side,
        price: Price,
        taker: &mut Order,
        trades: &mut Vec<Trade>,
    ) -> bool {
        let book = match maker_side {
            Side::Buy => &mut self.book.bids,
            Side::Sell => &mut self.book.asks,
        };

        let queue = match book.get_mut(&price) {
            Some(q) => q,
            None => return false,
        };

        while taker.quantity.0 > 0 {
            let maker = match queue.front_mut() {
                Some(m) => m,
                None => break,
            };

            let exec_qty = Quantity(taker.quantity.0.min(maker.quantity.0));

            trades.push(Trade {
                taker_order_id: taker.id,
                maker_order_id: maker.id,
                price,
                quantity: exec_qty,
                timestamp: taker.timestamp,
            });

            taker.quantity.0 -= exec_qty.0;
            maker.quantity.0 -= exec_qty.0;

            if maker.quantity.0 == 0 {
                let done = queue.pop_front().unwrap();
                self.book.order_index.remove(&done.id);
            }
        }

        if queue.is_empty() {
            book.remove(&price);
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
