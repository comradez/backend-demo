use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, get, web::{self, Bytes}};
use qstring::QString;
use diesel::{prelude::*, RunQueryDsl};
use serde_json::json;
use crate::Pool;
use crate::models::*;

#[get("/api/message")]
pub async fn get_message(request: HttpRequest, pool: web::Data<Pool>) -> impl Responder {
    use crate::schema::message::dsl::*;
    let db_connection = pool.get().unwrap();
    let query_string = request.query_string();
    let query_string  = QString::from(query_string);

    let limit = query_string.get("limit");
    let limit = match limit {
        Some(limit_str) => {
            let limit = limit_str.parse::<i32>();
            match limit {
                Ok(val) => val,
                Err(_e) => {
                    return HttpResponse::BadRequest().body("Field 'limit' is not a valid number");
                }
            }
        },
        None => 100
    };

    let offset = query_string.get("offset");
    let offset = match offset {
        Some(offset_str) => {
            let offset = offset_str.parse::<i32>();
            match offset {
                Ok(val) => val,
                Err(_e) => {
                    return HttpResponse::BadRequest().body("Field 'offset' is not a valid nu");
                }
            }
        },
        None => 400
    };
    
    let return_objects = message
        .order(id)
        .limit(limit as i64)
        .offset(offset as i64)
        .load::<PostMessage>(&db_connection).unwrap();
    let return_objects: Vec<String> = return_objects
        .into_iter()
        .map(|x| MessageJson::from(x))
        .map(|x| json!(x).to_string())
        .collect();
    let return_objects = json!(return_objects).to_string();
    HttpResponse::Ok().body(return_objects)
}

pub async fn get_post_message(request_raw: Bytes, request: HttpRequest) -> impl Responder {
    let name = request.cookie("user");
    let _name = match name {
        Some(cookie) => String::from(cookie.value()),
        None => String::from("Unknown")
    };
    //假装这里有数据库操作验证这个用户的身份
    if let Ok(text) = String::from_utf8(request_raw.to_vec()) {
        match serde_json::from_str::<MessageJson>(&text) {
            Ok(post_data) => {
                if post_data.title.len() > 100 {
                    return HttpResponse::BadRequest().body("Field 'title' Too Long");
                } else if post_data.content.len() > 400 {
                    return HttpResponse::BadRequest().body("Field 'content' Too Long");
                } else {
                    //假装这里有添加数据库内容的代码
                    return HttpResponse::Ok().body("Successfully created.");
                }
            },
            Err(_) => return HttpResponse::BadRequest().body("Json Parse Error")
        }
    }
    HttpResponse::BadRequest().body("Unknown Error")
}

#[get("/api/clearmessage")]
pub async fn clear_message() -> impl Responder {
    //假装这里操作了数据库清除了所有信息
    HttpResponse::Ok().body("Successfully cleared messages.")
}