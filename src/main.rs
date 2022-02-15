extern crate xmlrpc;

use crate::meta_weblog::cfg::{USER_INFO_CFG, BLOGS_INFO_CFG};
use std::fs::File;
use std::path::Path;
use std::fs;
use std::io::{stdout, stdin, Write};

use xmlrpc::{Request, Error};
use xmlrpc::Value;

mod meta_weblog;
use meta_weblog::weblog::{BlogInfo, Post, CategoryInfo};
use meta_weblog::rpc::MetaWeblog;
use meta_weblog::cfg::Config;
use crate::meta_weblog::weblog::WpCategory;


fn main() {
    let metaweblog = MetaWeblog::new("username".to_string(),
        "password".to_string(),
        "123".to_string());
    dbg!(metaweblog.get_categories().unwrap());

    // let p = metaweblog.get_post("15209798").unwrap();
    
    // //let postid = metaweblog.new_post(p, true).unwrap();
    // let mut category = WpCategory::default();
    // category.parent_id = -1;
    // category.name = "Test!!!".to_string();
    // let id = metaweblog.new_category(category).unwrap();
    // println!("{:#?}", id);
   
    //init_user_cfg("~/.cnblog/");
}

/// init user config
fn init_user_cfg(base_path: &str) -> Result<(), Error>{
    // Make sure the dictory exsits
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
    let user_path = path.join(USER_INFO_CFG);
    let user_path = user_path.as_path();
    if user_path.exists() {
        return Ok(());
    }
    // When false, we need to ask the user for their account and password
    let (username, password) = ask_question();
    Config::check_account(username.as_str(), password.as_str())?;
    
    // Check whether the master postid exists
    let num = Config::try_get_master_postid(&username, &password)?; 
    
    if num == 0 {
       // Now we need to create new blog info 
       let blogs_path = path.join(BLOGS_INFO_CFG);
       let blogs_path = blogs_path.as_path();
       Config::init_blogs_cfg(&username, &password, blogs_path);
    } else {
        todo!("download blogs_info file from remote cnblog");
    }
    Ok(())
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


