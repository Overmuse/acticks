use crate::api::Credentials;
use crate::Account;
use std::borrow::Borrow;

pub struct Simulator {
    account: Account
}

impl Simulator {
    pub fn new() -> Self {
        let creds = Credentials::new();
	let account = Account::from_creds(creds.clone());
	println!("{:?}", creds);
	Simulator {
	    account,
	}
    }

    pub fn get_account(&self) -> &Account {
	&self.account
    }
}
