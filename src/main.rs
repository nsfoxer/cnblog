extern crate xmlrpc;
extern crate filetime;

use crate::meta_weblog::cfg::{BLOGS_INFO_CFG, USER_INFO_CFG};
use std::fs;
use std::io::{stdin, stdout, Write};
use std::path::Path;


use filetime::FileTime;
use xmlrpc::{Error, Request};

mod meta_weblog;
use meta_weblog::cfg::{Config, Utility, BlogsInfoDO};
use meta_weblog::rpc::MetaWeblog;
use meta_weblog::weblog::{BlogInfo, CategoryInfo, Post};

fn main() {
    let metaweblog = MetaWeblog::new(
        "username".to_string(),
        "password".to_string(),
        "123".to_string(),
    );
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

fn _main() {
    let base_path_str = "config/";
    init_user_cfg(base_path_str);

    let blog_root_path_str = "articles/";

    // get user info
    let base_path = Path::new(base_path_str);
    let user_info = Config::read_user_info_cfg(&base_path.join(USER_INFO_CFG)).unwrap();

    // init config & weblog
    let mut cfg = Config::new(
        &user_info.username,
        &user_info.password,
        user_info.postid,
        &user_info.blogid,
        base_path_str,
    );
    cfg.init_conn(); // must call it 
    let mut weblog = MetaWeblog::new(
        user_info.username.to_string(),
        user_info.password.to_string(),
        user_info.blogid.to_string(),
    );

    // check blogs update
    if cfg.check_blogs_info_update() {
        // todo!("download remote new blog;");
        download_remote_new_blog(&cfg, &mut weblog, blog_root_path_str);
        //todo!("update remote changed blog");
        update_remote_changed_blog(&cfg, &mut weblog, blog_root_path_str);
        //todo!("move remote deleted blog;");
        delete_remote_changed_blog(&cfg, &mut weblog, blog_root_path_str);
        todo!("overwrite local blogs info");
    }
    todo!("update local changed blog and upload");
    todo!("update categories");
    todo!("update(save) local blogs info and upload;");
}

/// Save the corresponding blog according to the blogs_info
/// and change the modified timestamp of the blog at the same time
fn save_blogs_by_blogs_info(blogs_info: Vec<BlogsInfoDO>, weblog: &mut MetaWeblog, root_path: &str) {
    let path = Path::new(root_path);
    for blog_info in blogs_info {
        // 1. download
        let blog = weblog.get_post(blog_info.postid.to_string().as_str()).unwrap();

        // 2. save blog
        let blog_path = path.join(blog_info.blog_path.as_str());
        let dir_path = blog_path.parent().unwrap();
        if !dir_path.exists() {
            fs::create_dir_all(dir_path).unwrap();
        }
        fs::write(blog_path.as_path(), blog.description).unwrap();

        // 3. change file mtime
        Utility::modify_file_timestamp(blog_path.as_path(), blog_info.timestamp);
    }
}

/// compare local and remote info to download new blog
/// need to modify timestamp of new blog
fn download_remote_new_blog(cfg: &Config, weblog: &mut MetaWeblog, root_path: &str) {
    // 1. get new blogsinfo by comparing remote and local database
    let blogs_info = cfg.get_remote_new_blogs_info();
    
    // 2. download blog by postid and save
    println!("Info: find the following new file.");
    for blog_info in blogs_info.iter() {
        println!("file: {}", blog_info.blog_path);
    }
    save_blogs_by_blogs_info(blogs_info, weblog, root_path);
}

fn delete_remote_changed_blog(cfg: &Config, weblog: &mut MetaWeblog, root_path: &str) {
    // 1. get deleted blog by comparing remote and local database
    let blogs_info = cfg.get_remote_changed_blogs_info();

    // 2. delete(move) blog
    let deleted_root_path = Path::new("root_path").parent().unwrap().join("cnblog_deleted");
    println!("Warning: the following file will be moved to {}.", deleted_root_path.to_str().unwrap());
    for blog_info in blogs_info.iter() {
        println!("file: {}", blog_info.blog_path);
    }
    todo!("move blog");
    for blog_info in blogs_info.iter() {
        println!("file: {}", blog_info.blog_path);
    }
}

/// update changedblog by remote blog info
/// Note: it will overwrite older local blog
fn update_remote_changed_blog(cfg: &Config, weblog: &mut MetaWeblog, root_path: &str) {
    // 1. get changed blogsinfo by comparing remote and local database
    let blogs_info = cfg.get_remote_changed_blogs_info();

    // 2. save changed blog
    println!("Warning: the following file will be overwritten!");
    for blog_info in blogs_info.iter() {
        println!("file: {}", blog_info.blog_path);
    }
    save_blogs_by_blogs_info(blogs_info, weblog, root_path);
}

/// init user config
/// After this function is excuted, it will ensure that the configuration file exsits.
fn init_user_cfg(base_path: &str) -> Result<(), Error> {
    // Make sure the dictory exsits
    let base_path = Path::new(base_path);
    if base_path.exists() {
        if !base_path.is_dir() {
            panic!("user config base path should be a dictory");
        }
    } else {
        if let Err(e) = fs::create_dir_all(base_path) {
            panic!("Can't create dictory:{:?}\nError:{}", base_path, e);
        }
    }

    // Check whether the user information file exists
    let user_path = base_path.join(USER_INFO_CFG);
    let blogs_path = base_path.join(BLOGS_INFO_CFG);
    if user_path.exists() && blogs_path.exists() {
        return Ok(());
    }

    // When false, we need to ask the user for their account and password
    let (username, password) = ask_question();
    Config::check_account(username.as_str(), password.as_str())?;

    // Check whether the master postid exists
    let num = Config::try_get_master_postid(&username, &password)?;
    let blogs_path = base_path.join(BLOGS_INFO_CFG);
    let blogs_path = blogs_path.as_path();
    let mut postid = -1;
    if num == 0 {
        // Not exists
        // Now we need to create a new blog info
        Config::init_blogs_cfg(blogs_path);
        postid = Config::upload_new_blogs_cfg(&username, &password, blogs_path);
    } else {
        // Exists
        // Dowload BlogsInfo
        let cfg = Config::new(
            &username,
            &password,
            num,
            "123",
            base_path.to_str().unwrap(),
        );
        cfg.download_blogs_info();
        postid = num;
    }

    // Save user info
    Config::write_user_info_cfg(&username, &password, postid, base_path);
    Ok(())
}

/// ask username and password
fn ask_question() -> (String, String) {
    // 1. print a prompt
    println!(
        "The user info config file was not founded!\
            Now we need your username and password for cnblog web\
            (Press `Enter` confirm)"
    );

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
