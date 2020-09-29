use log::{debug, trace};
use reqwest::Client;
use tokio::stream::StreamExt;
use tokio::time::{DelayQueue, Duration};
struct PolygonMarket {
    subscribers: Vec<Recipient<TickerTrade>>,
    trades: Vec<TickerTrade>,
    queue: DelayQueue<TickerTrade>,
}

impl PolygonMarket {
    async fn download_data(&self, symbol: &str) -> Result<Vec<TickerTrade>, i32> {
        let client = Client::new();
        let url = format!(
            "https://api.polygon.io/v2/ticks/stocks/trades/{}/2020-09-18?apiKey={}",
            symbol,
            std::env::var("POLYGON_KEY").unwrap()
        );
        trace!("Making request: {}", &url);
        let req = client.get(&url).send().await.unwrap();
        let res = req.text().await.unwrap();
        let res: PolygonResponse = serde_json::from_str(&res).unwrap();
        Ok(res
            .results
            .iter()
            .cloned()
            .map(|t| TickerTrade(symbol.to_string(), t))
            .collect())
    }
}

impl Actor for PolygonMarket {
    type Context = Context<Self>;
}

impl Handler<Subscribe> for PolygonMarket {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, _ctx: Context<Self>) {
        self.subscribers.push(msg.0)
    }
}

impl Market for PolygonMarket {
    fn new(symbols: String, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        let mut trades = vec![];
        for symbol in symbols {
            debug!("Downloading data for {}", &symbol);
            let mut data = self.download_data(&symbol).await.unwrap();
            trades.append(&mut data);
        }
        trades.sort_by(|t1, t2| t2.1.timestamp.partial_cmp(&t1.1.timestamp).unwrap());
        PolygonMarket {
            subscribers: vec![],
            trades,
            queue: DelayQueue::new(),
        }
    }

    fn start(&mut self, rate: u64) {
        debug!("Scheduling trades");
        let first_message: TickerTrade = self.trades.pop().unwrap();
        let actual_now = Instant::now();
        let simulated_now = first_message.1.timestamp;
        self.queue
            .insert_at(first_message, actual_now + Duration::from_nanos(0));
        while let Some(message) = self.trades.pop() {
            log::trace!("Scheduling message: {:?}", &message);
            let event_time =
                actual_now + Duration::from_nanos((message.1.timestamp - simulated_now) / rate);
            self.queue.insert_at(message, event_time);
        }
    }
}
