use cnblog::Convert;

use std::collections::BTreeMap;

use iso8601::DateTime;
use xmlrpc::Value;

#[allow(non_snake_case)]
#[derive(Default, Debug, Convert)]
pub struct UserBlogs {
    app_key: String,
    username: String,
    password: String,
}

// BlogInfo
#[allow(non_snake_case)]
#[derive(Default, Debug, Convert)]
pub struct BlogInfo {
    pub blogid: String,
    pub url: String,
    pub blogName: String,
}

impl Post {
    //  It's used to write macro
    fn _convert2(&mut self, a: Value) {
        if let Value::Struct(a) = a {
            for (k, v) in a.into_iter() {
                if k == "dateCreated" {
                    if let Value::DateTime(v) = v {
                        self.dateCreated = v;
                        continue;
                    }
                    if let Value::Array(v) = v {
                        for v in v.into_iter() {
                            if let Value::String(v) = v {
                                self.categories.push(v);
                            }
                        }
                    }
                }
            }
        } else {
            panic!("das");
        }
    }
}

// Post
#[allow(non_snake_case)]
#[derive(Debug, Default, Clone, Convert)]
pub struct Post {
    pub postid: String,
    pub dateCreated: DateTime,
    pub description: String,
    pub title: String,
    pub categories: Vec<String>,
}

// CategoryInfo
#[allow(non_snake_case)]
#[derive(Debug, Default, Convert)]
pub struct CategoryInfo {
    pub description: String,
    pub htmlUrl: String,
    pub rssUrl: String,
    pub title: String,
    pub categoryid: String,
}

// WpCategory
#[derive(Debug, Default, Convert)]
pub struct WpCategory {
    pub name: String,
    pub parent_id: i32,
}
