struct AccountConfig {
    short_allowed: bool,
}

struct Position {

}

struct Order {

}

struct Account {
    id: String, // TODO: should be UUID
    cash: f64,
    positions: Vec<Position>,
    orders: Vec<Order>,
    config: AccountConfig
}

impl Account {
    get_account(self) -> Account {
        
    }
}

impl Account {
    fn get_positions(self) -> Vec<Position> {
        self.positions
    }

    fn get_orders(self) -> Vec<Order> {
        self.orders
    }
}

struct Simulator {
    account: Account,
}