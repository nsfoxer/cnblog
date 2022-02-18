extern crate xmlrpc;

use crate::meta_weblog::cfg::{BLOGS_INFO_CFG, USER_INFO_CFG};
use std::fs;
use std::io::{stdin, stdout, Write};
use std::path::Path;

use xmlrpc::Value;
use xmlrpc::{Error, Request};

mod meta_weblog;
use crate::meta_weblog::weblog::WpCategory;
use meta_weblog::cfg::Config;
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
    let weblog = MetaWeblog::new(
        user_info.username.to_string(),
        user_info.password.to_string(),
        user_info.blogid.to_string(),
    );

    // check blogs update
    if cfg.check_blogs_info_update() {
        todo!("download remote new blog;");
        todo!("update remote changed blog");
        todo!("move remote deleted blog;");
        todo!("overwrite local blogs info");
    }
    todo!("update local changed blog and upload");
    todo!("update categories");
    todo!("update(save) local blogs info and upload;");
}

/// compare local and remote info to download new blog
/// need to modify timestamp of new blog
fn download_remote_new_blog(cfg: &mut Config, root_path: &str) {
    cfg.get_new_blogs_info();
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
