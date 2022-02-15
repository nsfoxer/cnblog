extern crate xmlrpc;

use crate::meta_weblog::cfg::USER_INFO_CFG;
use std::fs::File;
use std::path::Path;
use std::fs;
use std::io::{stdout, stdin, Write};

use meta_weblog::rpc::MetaWeblog;
use xmlrpc::Request;
use xmlrpc::Value;

mod meta_weblog;
use meta_weblog::weblog::{BlogInfo, Post, CategoryInfo};

use crate::meta_weblog::weblog::WpCategory;


fn main() {
    let metaweblog = MetaWeblog::new("username".to_string(),
        "password".to_string(),
        "123".to_string());
    
    let p = metaweblog.get_post("15209798").unwrap();
    
    //let postid = metaweblog.new_post(p, true).unwrap();
    let mut category = WpCategory::default();
    category.parent_id = -1;
    category.name = "Test!!!".to_string();
    let id = metaweblog.new_category(category).unwrap();
    println!("{:#?}", id);
   
    init_user_cfg();
}

/// init user config
fn init_user_cfg(base_path: &str) {
    // Make sure the dictory exsts
    let path = Path::new(base_path);
    if path.exists() {
        if !path.is_dir() {
            panic!("user config base path should be a dictory");
        }
    } else {
        if let Err(e) = fs::create_dir_all(base_path) {
            panic!("Can't create dictory:{}\nError:{}", base_path, e);
        }
    }
    
    // Check whether the user information file exists
    let user_path = path.join(USER_INFO_CFG).as_path();
    if path.exists() {
        return;
    }
    // When false, we need to ask the user for their account and password
    let (username, password) = ask_question();
    check_account(username.as_str(), password.as_str());
}

/// ask username and password
fn ask_question() -> (String, String) {
    // 1. print a prompt
    println!("The user info config file was not founded!\
            Now we need your username and password for cnblog web\
            (Press `Enter` confirm)");

    let mut buf = String::new();

    // 2. get username
    print!("Please input your username: ");
    stdout().flush().unwrap();   
    stdin().read_line(&mut buf).unwrap();
    let username = buf.trim().to_string();

    // 3. get password
    print!("Please input your password: ");
    stdout().flush().unwrap();   
    stdin().read_line(&mut buf).unwrap();
    let password = buf.trim().to_string();
    
    (username, password)
}

fn check_account(username: &str, password: &str) {
    
}
