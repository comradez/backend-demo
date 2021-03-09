use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, get, web::{self, Bytes}};
use qstring::QString;
use diesel::{RunQueryDsl, insert_into, prelude::*};
use serde_json::json;
use chrono::prelude::*;
use crate::Pool;
use crate::models::*;

#[get("/api/message")]
pub async fn get_message(request: HttpRequest, pool: web::Data<Pool>) -> impl Responder {
    use crate::schema::message::dsl::*;
    let db_connection = pool.get().unwrap();
    let query_string = request.query_string();
    let query_string  = QString::from(query_string);

    let limit = query_string.get("limit");
    let limit: u32 = match limit {
        Some(limit) => 
            match limit.parse::<u32>() {
                Ok(val) => val, //有这个字段而且是合法正整数，获取这个值
                Err(_e) => return HttpResponse::BadRequest().body("Field 'limit' is not a valid number"),
            }, //有这个字段但不是合法正整数，返回400
        None => 100 //没有这个字段，默认为100
    };
    let offset = query_string.get("offset");
    let offset: u32 = match offset {
        Some(offset) =>
            match offset.parse::<u32>() {
                Ok(val) => val,
                Err(_e) => return HttpResponse::BadRequest().body("Field 'offset' is not a valid number"),
            },
        None => 400
    };
    let return_objects: Vec<String> = message
        .order(id)
        .limit(limit as i64)
        .offset(offset as i64)
        .load::<PostMessage>(&db_connection)
        .unwrap()
        .into_iter()
        .map(|x| MessageJson::from(x))
        .map(|x| json!(x).to_string())
        .collect(); //获取所有message，按照offset和limit进行筛选，然后将所有得到的PostMessage类型对象转换为MessageJson对象，然后Serialize
    let return_objects = json!(return_objects).to_string(); //把字符串vector序列化为json
    HttpResponse::Ok().body(return_objects)
}

pub async fn get_post_message(request_raw: Bytes, request: HttpRequest, pool: web::Data<Pool>) -> impl Responder {
    use crate::schema::user::dsl::*;
    let db_connection = pool.get().unwrap();
    let username = match request.cookie("user") {
        Some(cookie) => String::from(cookie.value()),
        None => String::from("Unknown")
    };
    let userid: i32 = match user.filter(name.eq(&username)).first::<PostUser>(&db_connection) {
        Ok(vec) => vec.id,
        Err(_) => return HttpResponse::BadRequest().body("User not exist!"),
    }; //验证用户的存在性，如果存在则得到用户的id
    if let Ok(text) = String::from_utf8(request_raw.to_vec()) {
        match serde_json::from_str::<MessageJson>(&text) {
            Ok(post_data) => {
                if post_data.title.len() > 100 {
                    return HttpResponse::BadRequest().body("Field 'title' Too Long");
                } else if post_data.content.len() > 400 {
                    return HttpResponse::BadRequest().body("Field 'content' Too Long");
                } else {
                    use crate::schema::message::dsl::*;
                    let new_object_id: i32 = match message
                        .order_by(id)
                        .first::<PostMessage>(&db_connection) {
                            Ok(item) => item.id + 1,
                            Err(_) => 1
                        }; //从数据库中获取最新的id，加1作为新的id
                    let new_object = PostMessage {
                        id: new_object_id,
                        user: userid,
                        title: post_data.title,
                        content: post_data.content,
                        pub_date: Local::now().naive_local(),
                    };
                    if let Err(_) = insert_into(message)
                        .values(new_object)
                        .execute(&db_connection) {
                            return HttpResponse::InternalServerError().body("Error Saving object");
                        }
                    //向数据库中添加内容
                    return HttpResponse::Ok().body("Successfully created.");
                }
            },
            Err(_) => return HttpResponse::BadRequest().body("Json Parse Error")
        }
    }
    HttpResponse::BadRequest().body("Unknown Error")
}

#[get("/api/clearmessage")]
pub async fn clear_message(pool: web::Data<Pool>) -> impl Responder {
    use crate::schema::message::dsl::*;
    let db_connection = pool.get().unwrap();
    match diesel::delete(message).execute(&db_connection) {
        Ok(_) => HttpResponse::Ok().body("Successfully cleared messages."),
        Err(_) => HttpResponse::InternalServerError().body("Error while deleting the table"),
    }
}