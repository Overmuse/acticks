use crate::credentials::Credentials;
use crate::Account;

#[derive(Clone)]
pub struct Simulator {
    pub account: Account
}

impl Simulator {
    pub fn new(creds: &Credentials) -> Self {
	let account = Account::from_creds(creds.clone());
	Simulator {
	    account,
	}
    }

    pub fn get_account(&self) -> Account {
	self.account.clone()
    }
}
