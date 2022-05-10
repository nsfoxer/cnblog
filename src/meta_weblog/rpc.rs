use crate::meta_weblog::weblog::WpCategory;
use crate::BlogInfo;
use crate::CategoryInfo;
use xmlrpc::{Error, Request, Value};

use super::weblog::Post;

const DELETE_POST: &str = "blogger.deletePost";
const EDIT_POST: &str = "metaWeblog.editPost";
const GET_CATEGORIES: &str = "metaWeblog.getCategories";
const GET_POST: &str = "metaWeblog.getPost";
const GET_RECENT_POSTS: &str = "metaWeblog.getRecentPosts";
const GET_USERS_BLOGS: &str = "blogger.getUsersBlogs";
const NEW_POST: &str = "metaWeblog.newPost";
const NEW_CATEGORY: &str = "wp.newCategory";
const SERVER_URL: &str = "https://rpc.cnblogs.com/metaweblog";

pub struct MetaWeblog {
    app_key: String,
    username: String,
    password: String,
    blogid: String,
    url: String,
}

impl MetaWeblog {
    // new
    pub fn new(username: String, password: String, blogid: String) -> Self {
        MetaWeblog {
            password,
            blogid,
            username: username.clone().to_string(),
            app_key: username.clone().to_string(),
            url: format!("{}/{}", SERVER_URL, username),
        }
    }

    pub fn new_post(&self, post: Post, publish: bool) -> Result<String, Error> {
        // 1. geerate arguments
        let mut arguments = Vec::<Value>::new();
        arguments.push(Value::String(self.blogid.to_string()));
        arguments.push(Value::String(self.username.to_string()));
        arguments.push(Value::String(self.password.to_string()));
        arguments.push(post.into());
        arguments.push(Value::Bool(publish));

        // 2. call rpc
        let result = self.rpc_request(NEW_POST, arguments)?;

        // 3. parse result
        if let Value::String(postid) = result {
            return Ok(postid);
        }
        Ok("-2".to_string())
    }

    pub fn new_category(&self, category: WpCategory) -> Result<i32, Error> {
        // 1. geerate arguments
        let mut arguments = Vec::<Value>::new();
        arguments.push(Value::String(self.blogid.to_string()));
        arguments.push(Value::String(self.username.to_string()));
        arguments.push(Value::String(self.password.to_string()));
        arguments.push(category.into());

        // 2. call rpc
        let result = self.rpc_request(NEW_CATEGORY, arguments)?;

        // 3. parse result
        if let Value::Int(categoryid) = result {
            return Ok(categoryid);
        }
        Ok(-1)
    }

    pub fn get_post(&self, postid: &str) -> Result<Post, Error> {
        // 1. geerate arguments
        let mut arguments = Vec::<Value>::new();
        arguments.push(Value::String(postid.to_string()));
        arguments.push(Value::String(self.username.to_string()));
        arguments.push(Value::String(self.password.to_string()));

        // 2. call rpc
        let result = self.rpc_request(GET_POST, arguments)?;

        // 3. parse result
        let post = Post::from(result);
        Ok(post)
    }

    pub fn get_recent_posts(&self, num: u32) -> Result<Vec<Post>, Error> {
        // 1. geerate arguments
        let mut arguments = Vec::<Value>::new();
        arguments.push(Value::String(self.blogid.to_string()));
        arguments.push(Value::String(self.username.to_string()));
        arguments.push(Value::String(self.password.to_string()));
        arguments.push(Value::Int(num as i32));

        // 2. call rpc
        let result = self.rpc_request(GET_RECENT_POSTS, arguments)?;

        // 3. parse result
        let mut posts = Vec::<Post>::new();
        if let Value::Array(results) = result {
            for v in results.into_iter() {
                posts.push(v.into());
            }
        }
        Ok(posts)
    }

    pub fn get_categories(&self) -> Result<Vec<CategoryInfo>, Error> {
        // 1. generate arguments
        let mut args = Vec::<Value>::new();
        args.push(Value::String(self.blogid.to_string()));
        args.push(Value::String(self.username.to_string()));
        args.push(Value::String(self.password.to_string()));

        // 2. call url
        let result = self.rpc_request(GET_CATEGORIES, args)?;

        // 3. parse result
        let mut categories = Vec::<CategoryInfo>::new();
        if let Value::Array(results) = result {
            for v in results.into_iter() {
                let category = CategoryInfo::from(v);
                categories.push(category);
            }
        }
        Ok(categories)
    }

    pub fn get_users_blogs(&self) -> Result<Vec<BlogInfo>, Error> {
        // 1. generate arguments
        let mut args = Vec::<Value>::new();
        args.push(Value::String(self.app_key.clone()));
        args.push(Value::String(self.username.clone()));
        args.push(Value::String(self.password.clone()));

        // 2. call rpc
        let result = self.rpc_request(GET_USERS_BLOGS, args)?;

        // 3. parse result
        let mut blog_infos = Vec::<BlogInfo>::new();
        if let Value::Array(results) = result {
            for v in results {
                let blog_info = BlogInfo::from(v);
                blog_infos.push(blog_info);
            }
        }
        Ok(blog_infos)
    }

    pub fn edit_post(&self, postid: &str, post: Post, publish: bool) -> Result<Value, Error> {
        // 1. generate parameters
        let mut arguments = Vec::<Value>::new();
        arguments.push(Value::String(postid.to_string()));
        arguments.push(Value::String(self.username.to_string()));
        arguments.push(Value::String(self.password.to_string()));
        arguments.push(post.into());
        arguments.push(Value::Bool(publish));

        // 2. call rpc
        let result = self.rpc_request(EDIT_POST, arguments)?;

        // 3. parse result
        Ok(result)
    }

    // "2022年 02月 10日 星期四 16:07:15 CST" 
    // Now the cnblog server could't delete articles by metaweblog
    // Error: Fault 0: 'Command cannot be issued to a replica: UNLINK blog_v2_BlogPosts-532134'
    #[deprecated]
    pub fn delete_post(&self, postid: &str, publish: bool) -> Result<bool, Error> {
        // 1. generate arguments
        let mut arguments = Vec::<Value>::new();
        arguments.push(Value::String(self.app_key.clone()));
        arguments.push(Value::String(postid.to_string()));
        arguments.push(Value::String(self.username.to_string()));
        arguments.push(Value::String(self.password.clone()));
        arguments.push(Value::Bool(publish));

        // 2. call rpc
        let result = self.rpc_request(DELETE_POST, arguments)?;

        // 3. parse result
        if let Value::Bool(v) = result {
            return Ok(v);
        }
        Ok(false)
    }

    fn rpc_request(&self, method: &str, args: Vec<Value>) -> Result<Value, Error> {
        // When `request` call `arg()`, owenership entry function. So we need rereceive
        let mut request = Request::new(method);

        for arg in args.into_iter() {
            request = request.arg(arg);
        }
        request.call_url(self.url.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::{WpCategory, MetaWeblog};
    #[test]
    fn new_category() {
        let weblog = MetaWeblog::new("nsfoxer".to_string(), "440EVxFSCXylKg".to_string(), "123".to_string());
        let mut category = WpCategory::default();
        category.name = "Cates".to_string();
        let a = weblog.new_category(category).unwrap();
        dbg!(a);
    }

    #[test]
    fn get_recent_posts() {
        let weblog = MetaWeblog::new("nsfoxer".to_string(), "440EVxFSCXylKg".to_string(), "123".to_string());
        let posts = weblog.get_recent_posts(100).unwrap();
        println!("{:?}", posts);
    }

    #[test]
    fn delete_post() {
        let weblog = MetaWeblog::new("nsfoxer".to_string(), "440EVxFSCXylKg".to_string(), "123".to_string());
        let posts = weblog.delete_post("16252136",true).unwrap();
        println!("{:?}", posts);
    }
}
