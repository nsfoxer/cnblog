extern crate filetime;
extern crate xmlrpc;

use chrono::Datelike;
use crate::meta_weblog::weblog::WpCategory;
use crate::meta_weblog::cfg::{BLOGS_INFO_CFG, USER_INFO_CFG};
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, create_dir};
use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::process::exit;

use chrono::Timelike;
use walkdir::{WalkDir, DirEntry};
use xmlrpc::Error;
use clap::Parser;
use dirs::config_dir;

mod meta_weblog;
use meta_weblog::cfg::{BlogsInfoDO, Config, Utility};
use meta_weblog::rpc::MetaWeblog;
use meta_weblog::weblog::{BlogInfo, CategoryInfo, Post};

/// It's a cnblog's blog (markdown) note synchronization tool.
#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    /// Root path of articles
    #[clap(short, long, default_value_t = String::from("articles"))]
    rootpath: String,
    
    /// Config directory of cnblog
    #[clap(short, long, default_value_t = String::from(config_dir().unwrap().join("cnblog").to_str().unwrap()))]
    config: String,
}

fn main() {
    let args = Args::parse();
    let base_path_str = args.config.as_str();
    if let Err(e) = init_user_cfg(base_path_str) {
        eprintln!("{e}");
        exit(1);
    }

    let blog_root_path_str = args.rootpath.as_str();

    // get user info
    let base_path = Path::new(base_path_str);
    let user_info = Config::read_user_info_cfg(&base_path.join(USER_INFO_CFG)).unwrap();

    // init config & weblog
    let mut cfg = Config::new(
        &user_info.username,
        &user_info.password,
        &user_info.app_key,
        user_info.postid,
        &user_info.blogid,
        base_path_str,
    );
    cfg.init_conn(); // must call it
    let mut weblog = MetaWeblog::new(
        user_info.username.to_string(),
        user_info.password.to_string(),
        user_info.blogid.to_string(),
        user_info.app_key.to_string(),
    );

    // check blogs update
    if cfg.check_blogs_info_update() {
        // todo!("download remote new blog;");
        download_remote_new_blog(&cfg, &mut weblog, blog_root_path_str);
        //todo!("update remote changed blog");
        update_remote_changed_blog(&cfg, &mut weblog, blog_root_path_str);
        //todo!("move remote deleted blog;");
        delete_remote_changed_blog(&cfg, &mut weblog, blog_root_path_str);
        //todo!("overwrite local blogs database");
        cfg = overwrite_local_blogs_database(cfg);
    }
    // update local blog after finishing syncing local and remote database(blogs info)
    // todo!("upload new blog")
    //todo!("update local changed blog and upload");
    //todo!("update categories");
    //todo!("update(save) local blogs info and upload;");
    sync_local_blogs_and_info(&cfg, &mut weblog, blog_root_path_str);
    //todo!("sync local database and local blogs"); ??????
    //todo!("upload local database");
    cfg.update_remote_database();
}

/// sync local blogs and local blogs info(database)
fn sync_local_blogs_and_info(cfg: &Config, weblog: &mut MetaWeblog, root_path: &str) {
    // 1. get local database blogs path
    let blogs_path = cfg.get_local_existed_blogs_path();
    let blogs_path: BTreeMap<String, ()> = blogs_path
        .into_iter()
        .map(|path| -> (String, ()) { (path, ()) })
        .collect();
    
    // 2. walk through a directory
    let blogs_info = cfg.get_local_existed_blogs_info();
    let mut fs_blogs_path = HashSet::new();
    for entry in WalkDir::new(root_path).into_iter().filter_entry(|e| is_not_hidden_and_is_markdown(e)){
        let entry = entry.unwrap();
        if entry.path().is_dir() {
            continue;
        }
        let local_path = entry.path().strip_prefix(root_path).unwrap().as_os_str().to_str().unwrap();
        let tlocal_path;
        if cfg!(target_family="windows") {
            tlocal_path = local_path.replace("\\", "/");
        } else {
            tlocal_path = local_path.to_string();
        }
        drop(local_path);
        fs_blogs_path.insert(tlocal_path.clone());

        // 2.1 upload new blog
        if !blogs_path.contains_key(tlocal_path.as_str()) {
            println!("Will upload new blog: {}", tlocal_path.as_str());
            upload_new_blog(&entry, &cfg, weblog, tlocal_path.as_str());
            continue;
        }

        // 2.2 get changed blog
        if let Some((old_timestamp, postid)) = blogs_info.get(tlocal_path.as_str()) {
            // local timestamp is greater than the remote timestamp
            let new_timestamp = Utility::get_file_timestamp(entry.path());
            if new_timestamp > *old_timestamp{
                println!("Will upload changed blog: {}", tlocal_path.as_str());
                update_local_blog(&entry, cfg, weblog, new_timestamp, *postid);
            }
        }
    }

    // 3. find deleted blogs and delete it
    for blog_path in blogs_info.keys() {
        if !fs_blogs_path.contains(blog_path) {
            if let Some((_, postid)) = blogs_info.get(blog_path) {
                let tblog_path;
                if cfg!(target_family="windows") {
                    tblog_path = blog_path.replace("/", "\\");
                } else {
                    tblog_path = blog_path.clone();
                }
                println!("Will delete(move) blog: {}", tblog_path.as_str());
                delete_blog(tblog_path.as_str(), cfg, weblog, *postid);
            }
        }
    }
}

/// Delete blog by postid and save info to database
fn delete_blog(blog_path: &str, cfg: &Config, weblog: &mut MetaWeblog, postid: i32) {
    // 1. delete remote blog
    println!("Warning: delete remote blog {}", blog_path);
    weblog.delete_post(postid.to_string().as_str(), true).unwrap();

    // 2. save database
    cfg.delete_post(postid);
}

/// update changed local blog
fn update_local_blog(entry: &DirEntry, cfg: &Config, weblog: &mut MetaWeblog, timestamp:i64, postid: i32) {
    // 1. generate basic post
    let content = fs::read_to_string(entry.path()).unwrap();
    let category = entry.path().parent().unwrap().file_name().unwrap().to_str().unwrap().to_string();
    let mut post = Post::default();
    post.description = content;
    post.categories.push(category);
    post.title = entry.path().file_name().unwrap().to_str().unwrap().to_string();

    // 2. upload changed category
    weblog.edit_post(postid.to_string().as_str(), post, true).unwrap();
    // 3. update database
    cfg.edit_post(postid, timestamp);
}

/// upload local new blog and save info to local database
fn upload_new_blog(entry: &DirEntry, cfg: &Config, weblog: &mut MetaWeblog, local_path: &str) {
    // 1. generate basic post
    let file_content = fs::read_to_string(entry.path()).unwrap();
    let timestamp = Utility::get_file_timestamp(entry.path());
    let category = entry.path().parent().unwrap().file_name().unwrap().to_str().unwrap().to_string();

    let mut post = Post::default();
    let now = chrono::Local::now();
    let s = format!("{}-{:02}-{:02}T{:02}:{:02}:{:02}", now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second());
    post.dateCreated = iso8601::datetime(s.as_str()).unwrap();
    post.description = file_content;
    post.categories.push(category.clone());
    post.categories.push("[Markdown]".to_string());
    post.title = entry.path().file_name().unwrap().to_str().unwrap().to_string();

    // 2. check category else upload new category
    let categories = cfg.get_local_categories();
    if !categories.contains(&category) {
        println!("New category: {}", category);
        // insert new category and upload category
        cfg.new_category(&category);
        let mut cate = WpCategory::default();
        cate.name = category.clone();
        weblog.new_category(cate).unwrap();
        //categories.insert(category);
    }

    // 3. update database
    let postid = weblog.new_post(post, true).unwrap();
    cfg.new_post(local_path, postid.parse().unwrap(), timestamp);
}

/// if entry is not hidden and extension is markdown, return true, otherwise false;
fn is_not_hidden_and_is_markdown(entry: &DirEntry) -> bool {
    // 1. entry is not hidden
    match entry.file_name().to_str() {
        Some(e) => {
            if e.starts_with(".") {
                return false;
            }
        },
        None => return false,
    }
    // 2. get entry suffix
    let path = entry.path();
    if path.is_dir() {
        return true;
    }
    let suffix = match path.extension() {
        Some(ext) =>  ext.to_str().unwrap_or(""),
        None => "",
    };
    if suffix != "md" && suffix != "markdown" {
        return false;
    }
    return true;
}

/// overwrite local blogs database
fn overwrite_local_blogs_database(cfg: Config) -> Config {
    cfg.overwrite_local_database()
}

/// Save the corresponding blog according to the blogs_info
/// and change the modified timestamp of the blog at the same time
fn save_blogs_by_blogs_info(
    blogs_info: Vec<BlogsInfoDO>,
    weblog: &mut MetaWeblog,
    root_path: &str,
) {
    let path = Path::new(root_path);
    for blog_info in blogs_info {
        // 1. download
        let blog = weblog
            .get_post(blog_info.postid.to_string().as_str())
            .unwrap();

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
    let mut blogs_info = cfg.get_remote_new_blogs_info();
    let blogs_info2 = cfg.get_local_lost_blogs_info(root_path);
    blogs_info.extend(blogs_info2);

    // 2. download blog by postid and save
    println!("Info: find the following new file.");
    for blog_info in blogs_info.iter() {
        println!("file: {}", blog_info.blog_path);
    }
    save_blogs_by_blogs_info(blogs_info, weblog, root_path);
}

/// delete(move) file from root_path to delete_path
fn delete_blogs_by_blogs_info(blogs_info: Vec<BlogsInfoDO>, root_path: &str, delete_path: &str) {
    // 1. determine whether the delete path exists
    let delete_path = Path::new(delete_path);
    if !delete_path.exists() {
        create_dir(delete_path).unwrap();
    }
    if !delete_path.is_dir() {
        eprintln!(
            "{:?} already exists but is not a dictory",
            delete_path.to_str()
        );
        panic!(
            "{:?} already exists but is not a dictory",
            delete_path.to_str()
        );
    }

    // 2. move file to delete path with postid name
    // the new and old file need to be in same mount point
    let root_path = Path::new(root_path);
    for blog_info in blogs_info {
        let old_path = root_path.join(blog_info.blog_path);
        let new_path = delete_path
            .join(blog_info.postid.to_string() + old_path.file_name().unwrap().to_str().unwrap());
        if let Err(e) = fs::rename(old_path.as_path(), new_path) {
            eprintln!(
                "Warning: a error occurred while moving {:?} to {:?}. Error: {} ",
                old_path, delete_path, e
            );
        }
    }
}

/// compare local and remote info to delete old blog
/// Note: old blog will be moved to delete dir
fn delete_remote_changed_blog(cfg: &Config, weblog: &mut MetaWeblog, root_path: &str) {
    // 1. get deleted blog by comparing remote and local database
    let blogs_info = cfg.get_remote_deleted_blogs_info();

    // 2. delete(move) blog
    let deleted_root_path = Path::new(root_path)
        .join(".cnblog_deleted");
    println!(
        "Warning: the following file will be moved to {}.",
        deleted_root_path.to_str().unwrap()
    );
    for blog_info in blogs_info.iter() {
        weblog.delete_post(blog_info.postid.to_string().as_str(), true).unwrap();
        println!("file: {}", blog_info.blog_path);
    }
    delete_blogs_by_blogs_info(blogs_info, root_path, deleted_root_path.to_str().unwrap());
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
    let (username, password, app_key) = ask_question();
    Config::check_account(username.as_str(), password.as_str(), app_key.as_str())?;

    // Check whether the master postid exists
    let num = Config::try_get_master_postid(&username, &password, &app_key)?;
    let blogs_path = base_path.join(BLOGS_INFO_CFG);
    let blogs_path = blogs_path.as_path();
    let postid;
    if num == 0 {
        // Not exists
        // Now we need to create a new blog info
        Config::init_blogs_cfg(blogs_path).unwrap();
        postid = Config::upload_new_blogs_cfg(&username, &password, &app_key, blogs_path);
    } else {
        // Exists
        // Dowload BlogsInfo
        let cfg = Config::new(
            &username,
            &password,
            &app_key,
            num,
            "123",
            base_path.to_str().unwrap(),
        );
        cfg.download_blogs_info();
        cfg.force_increase_timestamp_to_download_blogs();
        postid = num;
    }

    // Save user info
    Config::write_user_info_cfg(&username, &password, &app_key, postid, &user_path);
    Ok(())
}

/// ask username and password and app_key
fn ask_question() -> (String, String, String) {
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
    buf.clear();
    stdin().read_line(&mut buf).unwrap();
    let password = buf.trim().to_string();

    // 4. get app_key
    print!("Please input your app_key: ");
    stdout().flush().unwrap();
    buf.clear();
    stdin().read_line(&mut buf).unwrap();
    let app_key = buf.trim().to_string();

    println!("");

    (username, password, app_key)
}
