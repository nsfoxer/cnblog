use super::rpc::MetaWeblog;
use xmlrpc::Error;


pub const USER_INFO_CFG: &str = "user_info.json";

pub struct Config {
    
}

impl Config {
    /// check username and password valid!
    /// Return Error while user info is wrong, else return 
    pub fn check_account(username: &str, password: &str) -> Result<(), Error>{
        let mut weblog = MetaWeblog::new(username.to_string(),
            password.to_string(), "123".to_string());
        weblog.get_users_blogs()?;
        Ok(())
    }

    pub fn () {
        
    }
}