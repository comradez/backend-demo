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
                Err(_) => return HttpResponse::BadRequest().body(format!("{} is not a number", limit)),
            }, //有这个字段但不是合法正整数，返回400
        None => 100 //没有这个字段，默认为100
    };
    let offset = query_string.get("offset");
    let offset: u32 = match offset {
        Some(offset) =>
            match offset.parse::<u32>() {
                Ok(val) => val,
                Err(_) => return HttpResponse::BadRequest().body(format!("{} is not a number", offset)),
            },
        None => 0
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
    HttpResponse::Ok().json(return_objects)
}

pub async fn get_post_message(request_raw: Bytes, request: HttpRequest, pool: web::Data<Pool>) -> impl Responder {
    use crate::schema::user::dsl::*;
    let db_connection = pool.get().unwrap();
    let username = match request.cookie("user") {
        Some(cookie) => String::from(cookie.value()),
        None => String::from("Unknown")
    };
    if username.len() > 20 {
        return HttpResponse::BadRequest().body("User name too long");
    } //验证用户名长度合法
    let message_user = match user.filter(name.eq(&username)).first::<PostUser>(&db_connection) {
        Ok(vec) => vec,
        Err(_) => { //先尝试插入一个
            let new_user_id = match user
                .order_by(id.desc())
                .first::<PostUser>(&db_connection) {
                    Ok(item) => item.id + 1,
                    Err(_) => 1,
                };
            let new_user = PostUser {
                id: new_user_id,
                name: username.clone(),
                register_date: Local::now().naive_local(),
            };
            if let Err(_e) = insert_into(user)
                .values(&new_user)
                .execute(&db_connection) {
                    return HttpResponse::BadRequest().body("Validation Error of user:");
                }
            new_user
        }
    }; //验证用户的存在性，如果存在则得到用户，否则尝试创建
    if let Ok(text) = String::from_utf8(request_raw.to_vec()) {
        match serde_json::from_str::<ReceiveMessageJson>(&text) {
            Ok(post_data) => {
                if post_data.title.len() > 100 {
                    return HttpResponse::BadRequest().body("Field 'title' Too Long");
                } else if post_data.content.len() > 400 {
                    return HttpResponse::BadRequest().body("Field 'content' Too Long");
                } else {
                    use crate::schema::message::dsl::*;
                    let new_object_id: i32 = match message
                        .order_by(id.desc())
                        .first::<PostMessage>(&db_connection) {
                            Ok(item) => item.id + 1,
                            Err(_) => 1
                        }; //从数据库中获取最新的id，加1作为新的id
                    let new_object = PostMessage {
                        id: new_object_id,
                        user: message_user.id,
                        title: post_data.title,
                        content: post_data.content,
                        pub_date: Local::now().naive_local(),
                    };
                    if let Err(_e) = insert_into(message)
                        .values(new_object)
                        .execute(&db_connection) {
                            return HttpResponse::InternalServerError().body("Error Saving object");
                        }
                    //向数据库中添加内容
                    return HttpResponse::Created().body("message was sent successfully");
                }
            },
            Err(_) => return HttpResponse::BadRequest().body("Json Parse Error")
        }
    }
    HttpResponse::InternalServerError().body("Unknown Error")
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