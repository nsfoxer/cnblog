use std::path::PathBuf;
use std::{path::Path, io::Read};
use std::fs::File;
use std::io::Write;

use regex::Regex;
use base64;
use xmlrpc::Error;
use rusqlite::{Connection, OpenFlags, params};
use chrono::prelude::*;


use super::rpc::MetaWeblog;
use super::weblog::{Post, WpCategory};

pub const BLOGS_INFO_CFG: &str = "blogs_info.sqlite";
pub const USER_INFO_CFG: &str = "user_info.json";

const MASTER_BLOGS_CFG: &str = "MASTER_CNBLOG_BLOGS_INFO_CFG";

pub struct Config {
    master_postid: i32,

    blogs_info_cfg_path:  PathBuf,
    weblog: MetaWeblog,
}

impl Config {
    /// create a new Config
    pub fn new(username: &str, password: &str, master_postid: i32, base_path: &str) -> Self {
        let blogs_path = PathBuf::from(base_path).join(MASTER_BLOGS_CFG);
        let weblog = MetaWeblog::new(
            username.to_string(), 
            password.to_string(),
            "123".to_string());
        Config {
            weblog,
            master_postid,
            blogs_info_cfg_path: blogs_path
        }
    }

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

    pub fn upload_new_blogs_cfg(username: &str, password: &str, blogs_path: &Path) -> i32 {
        // 1. get a new postid for blogs 
        let weblog = MetaWeblog::new(
            username.to_string(), password.to_string(), "123".to_string());
        let mut post = Post::default();
        post.title = "[CNBLOG]BLOGS_INFO_CFG".to_string();
        let postid: i32 = weblog.new_post(post, false).unwrap().parse().unwrap();

        // 2. upload new category
        let category =  format!("{}[CNBLOG]", postid);
        let mut wp_category = WpCategory::default();
        wp_category.name = category.clone();
        wp_category.parent_id = -1;
        weblog.new_category(wp_category).unwrap();

        // 3. update local database
        let now = Local::now().timestamp();
        let conn = Connection::open_with_flags(blogs_path, OpenFlags::SQLITE_OPEN_READ_WRITE).unwrap();
        conn.execute("\
            insert into BlogsInfo (blog_path, postid, datetime)\
            values (?, ?, ?)",
     params![MASTER_BLOGS_CFG, postid, now]);
        drop(conn);

        // 4. upload database
        post.description = Config::file2base64(blogs_path);
        post.categories.push(category);
        weblog.edit_post(postid.to_string().as_str(), post, false).unwrap();

        postid
    }

    /// download blogs from 
    pub fn download_blogs_info(&self) {
        self.download_blogs_info();
    }
    fn download_blogs_info_to_path(&self, path: &Path) {
        // 1. download blogs info 
        let post = self.weblog.get_post(self.master_postid.to_string().as_str()).unwrap();
        
        // 2. decode and save
        Config::base642file(post.description.as_str(), path);
    }

    /// convert file to base64 string
    fn file2base64(file_path: &Path) -> String{
        // 1. read content
        let f = File::open(file_path).unwrap(); 
        let mut buffer = Vec::<u8>::new();
        f.read_to_end(&mut buffer).unwrap();

        // 2. base64 for content
        let base = base64::encode(buffer);
        base
        
    }

    /// convert base64 to file 
    fn base642file(base: &str, file_path: &Path) {
        // 1.decode base64
        let bytes = base64::decode(base).unwrap();

        // 2. write file
        let mut f = File::create(file_path).unwrap();
        f.write_all(&bytes).unwrap();
        
    }
}
