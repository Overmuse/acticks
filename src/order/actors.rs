use super::types::{Order, OrderStatus};
use crate::exchange::TradeFill;
use actix::prelude::*;
use log::debug;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "HashMap<Uuid, Order>")]
pub struct GetOrders;

#[derive(Default)]
pub struct OrderManager {
    pub orders: HashMap<Uuid, Order>,
}

impl Actor for OrderManager {
    type Context = Context<Self>;
}

impl actix::Supervised for OrderManager {}

impl SystemService for OrderManager {
    fn service_started(&mut self, _ctx: &mut Context<Self>) {
        debug!("OrderManager service started");
    }
}

impl Handler<GetOrders> for OrderManager {
    type Result = MessageResult<GetOrders>;

    fn handle(&mut self, _msg: GetOrders, _ctx: &mut Context<Self>) -> Self::Result {
        MessageResult(self.orders.clone())
    }
}

#[derive(Message)]
#[rtype(result = "Option<Order>")]
pub struct GetOrderByClientOrderId {
    pub client_order_id: String,
}

impl Handler<GetOrderByClientOrderId> for OrderManager {
    type Result = Option<Order>;

    fn handle(&mut self, msg: GetOrderByClientOrderId, _ctx: &mut Context<Self>) -> Self::Result {
        self.orders
            .values()
            .find(|order| order.client_order_id == msg.client_order_id)
            .cloned()
    }
}

#[derive(Message)]
#[rtype(result = "Option<Order>")]
pub struct GetOrderById {
    pub id: Uuid,
}

impl Handler<GetOrderById> for OrderManager {
    type Result = Option<Order>;

    fn handle(&mut self, msg: GetOrderById, _ctx: &mut Context<Self>) -> Self::Result {
        self.orders.get(&msg.id).cloned()
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PostOrder {
    pub order: Order,
}

impl Handler<PostOrder> for OrderManager {
    type Result = ();

    fn handle(&mut self, msg: PostOrder, _ctx: &mut Context<Self>) -> Self::Result {
        self.orders.insert(msg.order.id, msg.order);
    }
}

impl Handler<TradeFill> for OrderManager {
    type Result = ();

    fn handle(&mut self, msg: TradeFill, _ctx: &mut Context<Self>) -> Self::Result {
        self.orders.entry(msg.order.id).and_modify(|order| {
            let time = Some(msg.time);
            order.filled_qty = order.qty;
            order.updated_at = time;
            order.filled_at = time;
            order.filled_avg_price = Some(msg.price);
            order.status = OrderStatus::Filled;
        });
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct CancelOrders;

impl Handler<CancelOrders> for OrderManager {
    type Result = ();

    fn handle(&mut self, _msg: CancelOrders, _ctx: &mut Context<Self>) -> Self::Result {
        for order in self.orders.values_mut() {
            match order.status {
                OrderStatus::Filled | OrderStatus::Expired | OrderStatus::Canceled => (),
                _ => order
                    .cancel()
                    .expect("All other statuses should be cancelable"),
            }
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct CancelOrder(pub Uuid);

impl Handler<CancelOrder> for OrderManager {
    type Result = ();

    fn handle(&mut self, msg: CancelOrder, _ctx: &mut Context<Self>) -> Self::Result {
        self.orders.get_mut(&msg.0).unwrap().cancel().unwrap();
    }
}