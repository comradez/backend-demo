use serde::{Deserialize, Serialize};
use crate::schema::*;
use chrono;

#[derive(Debug, Insertable, Queryable)]
#[table_name = "message"]
pub struct PostMessage {
    pub id: i32,
    pub user: i32,
    pub title: String,
    pub content: String,
    pub pub_date: chrono::NaiveDateTime,
} //用来与数据库进行交互的结构体

impl From<PostMessage> for MessageJson {
    fn from(item: PostMessage) -> Self {
        MessageJson {
            id: item.id,
            user: item.user,
            title: item.title,
            content: item.content,
            pub_date: item.pub_date.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageJson {
    pub id: i32,
    pub user: i32,
    pub title: String,
    pub content: String,
    pub pub_date: String,
} //用来转Json的结构体，虽然没有明白和Message分开有什么必要……

#[derive(Debug, Insertable, Queryable)]
#[table_name = "user"]
pub struct PostUser {
    pub id: i32,
    pub name: String,
    pub register_date: chrono::NaiveDateTime,
}

impl From<PostUser> for UserJson {
    fn from(item: PostUser) -> Self {
        UserJson {
            id: item.id,
            name: item.name,
            register_date: item.register_date.to_string()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserJson {
    pub id: i32,
    pub name: String,
    pub register_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReceiveMessageJson {
    pub title: String,
    pub content: String,
}