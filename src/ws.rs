use actix::prelude::*;
use actix::Addr;
use actix_web_actors::ws;

#[derive(Default)]
pub struct Server {
    pub sessions: Vec<Recipient<Message>>,
}

impl Server {
    fn send_message(&self, message: &str) {
        for session in &self.sessions {
            let _ = session.do_send(Message(message.to_owned()));
        }
    }
}

impl Actor for Server {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Message)]
#[rtype(result = "()")]
struct Connect(Recipient<Message>);

impl Handler<Connect> for Server {
    type Result = ();

    fn handle(&mut self, msg: Connect, _ctx: &mut Context<Self>) {
        self.sessions.push(msg.0);
    }
}
