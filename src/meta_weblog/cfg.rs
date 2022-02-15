use std::path::Path;

use regex::Regex;
use xmlrpc::Error;
use rusqlite::{Connection};


use super::rpc::MetaWeblog;

pub const USER_INFO_CFG: &str = "user_info.json";
pub const BLOGS_INFO_CFG: &str = "blogs_info.sqlite";

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

    /// try get master postid which that cantians blogs info
    pub fn try_get_master_postid(username: &str, password: &str) -> Result<i32, Error>{
        let weblog = MetaWeblog::new(username.to_string(),
            password.to_string(), "123".to_string());
        let categories = weblog.get_categories()?;

        // get "[随笔分类]%d[CNBLOG]" postid
        let reg = Regex::new(r"[随笔分类](\d)+[CNBLOG]").unwrap();
        for category in categories {
            if reg.is_match(category.title.as_str()) {
                let num = reg.captures(category.title.as_str())
                    .unwrap().get(0).unwrap();
                let num: i32 = num.as_str().parse().unwrap();
                return Ok(num);
            }
        }
        Ok(0)
    }

    /// init blogs cfg
    pub fn init_blogs_cfg(username: &str, password: &str, blogs_path: &Path) {
        if blogs_path.exists() {
            eprintln!("blogs_path should be not exists! But it's existed!");
            return;
        }
        Config::create_database(blogs_path);
    }

    /// create database about blogs info in database_path
    /// Any error will panic (unwrap)
    fn create_database(database_path: &Path) {
        // create database
        let conn = Connection::open(database_path).unwrap();
        
        // create table
        conn.execute(
            "create table BlogsInfo (
                id integer primary key, -- primary key
                blog_path nvarchar,  -- local blog path
                postid integer,      -- postid of remote corresponding blog
                datetime integer,    -- last upload timestamp
            );
            create table Category (
                id integer primary key, -- primary key (be meaningless)
                category nvarchar,      --  category name
            );", []).unwrap();
    }
}
