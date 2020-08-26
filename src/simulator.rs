use crate::api::Credentials;
use crate::Account;

pub struct Simulator {
    accounts: Vec<Account>
}

impl Simulator {
    pub fn new() -> Self {
	Simulator {
	    accounts: vec!(),
	}
    }

    pub fn create_account(&mut self) -> Credentials {
        let creds = Credentials::new();
	let account = Account::from_creds(creds.clone());
	self.accounts.push(account);
	println!("{:?}", self.accounts);
	creds
    }
}
