use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::{io::Read, path::Path};

use base64;
use chrono::prelude::*;
use regex::Regex;
use rusqlite::{params, Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use tempfile::{tempfile, NamedTempFile};
use xmlrpc::Error;

use super::rpc::MetaWeblog;
use super::weblog::{Post, WpCategory};

pub const BLOGS_INFO_CFG: &str = "blogs_info.sqlite";
pub const USER_INFO_CFG: &str = "user_info.json";

const MASTER_BLOGS_CFG: &str = "MASTER_CNBLOG_BLOGS_INFO_CFG";

/// user info config
#[derive(Serialize, Deserialize, Debug)]
pub struct UserInfo {
    pub username: String,
    pub password: String,
    pub blogid: String,
    pub postid: i32,
}

pub struct Config {
    master_postid: i32,

    blogs_info_cfg_path: PathBuf,
    weblog: MetaWeblog,
    temp_data_file: NamedTempFile,
    local_conn: Connection,
    cnblog_conn: Connection,
}

impl Config {
    /// create a new Config
    pub fn new(
        username: &str,
        password: &str,
        master_postid: i32,
        blogid: &str,
        base_path: &str,
    ) -> Self {
        let blogs_path = PathBuf::from(base_path).join(MASTER_BLOGS_CFG);
        let weblog = MetaWeblog::new(
            username.to_string(),
            password.to_string(),
            blogid.to_string(),
        );

        Config {
            weblog,
            master_postid,
            blogs_info_cfg_path: blogs_path,
            temp_data_file: NamedTempFile::new().unwrap(),
            local_conn: Connection::open_in_memory().unwrap(),
            cnblog_conn: Connection::open_in_memory().unwrap(),
        }
    }

    /// check username and password valid!
    /// Return Error while user info is wrong, else return
    pub fn check_account(username: &str, password: &str) -> Result<(), Error> {
        let mut weblog = MetaWeblog::new(
            username.to_string(),
            password.to_string(),
            "123".to_string(),
        );
        weblog.get_users_blogs()?;
        Ok(())
    }

    /// try get master postid which that cantians blogs info
    pub fn try_get_master_postid(username: &str, password: &str) -> Result<i32, Error> {
        let weblog = MetaWeblog::new(
            username.to_string(),
            password.to_string(),
            "123".to_string(),
        );
        let categories = weblog.get_categories()?;

        // get "[随笔分类]%d[CNBLOG]" postid
        let reg = Regex::new(r"[随笔分类](\d)+[CNBLOG]").unwrap();
        for category in categories {
            if reg.is_match(category.title.as_str()) {
                let num = reg
                    .captures(category.title.as_str())
                    .unwrap()
                    .get(0)
                    .unwrap();
                let num: i32 = num.as_str().parse().unwrap();
                return Ok(num);
            }
        }
        Ok(0)
    }

    /// init blogs cfg
    pub fn init_blogs_cfg(blogs_path: &Path) {
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
                timestamp integer,    -- last upload timestamp
                deleted BOOLEAN not null check (deleted in (0, 1)), -- whether is deleted        

            );
            create table Category (
                id integer primary key, -- primary key (be meaningless)
                category nvarchar,      -- category name
                num integer,            -- number of category used
            );",
            [],
        )
        .unwrap();
    }

    /// Upload a new blogs config file
    /// Will get a new postid for blogs info and generate a new category with postid
    pub fn upload_new_blogs_cfg(username: &str, password: &str, blogs_path: &Path) -> i32 {
        // 1. get a new postid for blogs
        let weblog = MetaWeblog::new(
            username.to_string(),
            password.to_string(),
            "123".to_string(),
        );
        let mut post = Post::default();
        post.title = "[CNBLOG]BLOGS_INFO_CFG".to_string();
        let postid: i32 = weblog
            .new_post(post.clone(), false)
            .unwrap()
            .parse()
            .unwrap();

        // 2. upload new category
        let category = format!("{}[CNBLOG]", postid);
        let mut wp_category = WpCategory::default();
        wp_category.name = category.clone();
        wp_category.parent_id = -1;
        weblog.new_category(wp_category).unwrap();

        // 3. update local database
        let now = Local::now().timestamp();
        let conn =
            Connection::open_with_flags(blogs_path, OpenFlags::SQLITE_OPEN_READ_WRITE).unwrap();
        conn.execute(
            "\
            insert into BlogsInfo (blog_path, postid, datetime)\
            values (?, ?, ?, ?)",
            params![MASTER_BLOGS_CFG, postid, now, 1],
        );
        drop(conn); // Saved database to upload file

        // 4. upload database
        post.description = Config::file2base64(blogs_path);
        post.categories.push(category);
        weblog
            .edit_post(postid.to_string().as_str(), post, false)
            .unwrap();

        postid
    }

    /// download blogs from cnblog to blogs_path
    pub fn download_blogs_info(&self) {
        self.download_blogs_info_to_path(self.blogs_info_cfg_path.as_path());
    }
    fn download_blogs_info_to_path(&self, path: &Path) {
        // 1. download blogs info
        let post = self
            .weblog
            .get_post(self.master_postid.to_string().as_str())
            .unwrap();

        // 2. decode and save
        Config::base642file(post.description.as_str(), path);
    }

    /// convert file to base64 string
    fn file2base64(file_path: &Path) -> String {
        // 1. read content
        let mut f = File::open(file_path).unwrap();
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

    /// Write user basic info
    pub fn write_user_info_cfg(username: &str, password: &str, postid: i32, user_info_path: &Path) {
        if user_info_path.exists() {
            println!(
                "The {:?} file already exists!!!\nI'will overwrite it!",
                user_info_path
            );
        }
        // Get real blogid by using a fake value
        let weblog = MetaWeblog::new(
            username.to_string(),
            password.to_string(),
            "123".to_string(),
        );
        let userblogs = weblog.get_users_blogs().unwrap();
        let userblog = userblogs.get(0).unwrap();
        let blogid = userblog.blogid.clone();

        // Serialize User Info
        let user_info = UserInfo {
            username: username.to_string(),
            password: password.to_string(),
            postid,
            blogid,
        };
        let serialize = serde_json::to_string(&user_info).unwrap();

        // Write user info path
        fs::write(user_info_path, serialize).expect("Unable to write file for user_info");
    }

    /// Read user basic info
    pub fn read_user_info_cfg(user_info_path: &Path) -> Option<UserInfo> {
        // check the legitimacy of user information path
        if !user_info_path.exists() {
            return None;
        }

        // read user information
        let deserialization = fs::read_to_string(user_info_path).unwrap();

        // convert user information
        let user_info =
            serde_json::from_str(&deserialization).expect("Unable to parse user info file!");
        Some(user_info)
    }

    /// check blogs info for updates
    pub fn check_blogs_info_update(&self) -> bool {
        let local_timestamp: i32 = self
            .local_conn
            .query_row(
                "
            select timestamp from BlogsInfo where postid = ?
            ",
                [self.master_postid],
                |row| row.get(0),
            )
            .unwrap();
        let remote_timestamp: i32 = self
            .cnblog_conn
            .query_row(
                "
            select timestamp from BlogsInfo where postid = ?",
                [self.master_postid],
                |row| row.get(0),
            )
            .unwrap();
        if remote_timestamp > local_timestamp {
            return true;
        }
        if local_timestamp > remote_timestamp {
            panic!("local blogs info is newer than remote");
        }
        return false;
    }

    /// init Config loalc and remote Conn
    pub fn init_conn(&mut self) {
        // 1. init local conn
        self.local_conn = Connection::open(self.blogs_info_cfg_path.as_path()).unwrap();

        // 2. download blogs info
        self.download_blogs_info_to_path(self.temp_data_file.path());

        // 3. init remote blogs conn
        self.cnblog_conn = Connection::open(self.temp_data_file.path()).unwrap();
    }

    pub fn get_new_blogs_info(&self) -> Vec<BlogsInfoDO>{
        // 1. query local and remote blogs
        let local_blogs = self.query_blogs_info_do(&self.local_conn);
        let remote_blogs = self.query_blogs_info_do(&self.cnblog_conn);
        
        // 2. compare
        let new_blogs: Vec<BlogsInfoDO> = remote_blogs.into_iter()
            .filter(|(postid, _)| !local_blogs.contains_key(postid))
            .map(|(_, blog)|->BlogsInfoDO {blog})
            .collect();
        return new_blogs;
    }

    fn query_blogs_info_do(&self, conn: &Connection) -> BTreeMap<i32, BlogsInfoDO> {
        // 1. prepare sql
        let sql = "\
            select blog_path, postid, timestamp \
            from BlogsInfo where deleted = 0";
        let mut stmt = self.local_conn.prepare(sql).unwrap();
        
        // 2. get info
        let blogs = stmt
            .query_map([], |row| {
                Ok(BlogsInfoDO {
                    blog_path: row.get(0).unwrap(),
                    postid: row.get(1).unwrap(),
                    timestamp: row.get(2).unwrap(),
                    deleted: false,
                })
            }).unwrap();
        
        // 3. construct BTreeMap
        let mut btmap = BTreeMap::new();
        blogs.for_each(|blog| {
            let blog = blog.unwrap();
            btmap.insert(blog.postid, blog);
        });
        return btmap;
    }
}

pub struct BlogsInfoDO {
    pub blog_path: String,
    pub postid: i32,
    pub timestamp: i64,
    pub deleted: bool,
}
