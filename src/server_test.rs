#[cfg(test)]
mod server_test {
    use actix_web::{App, test::{self}, web::{Bytes}, dev::{Body, ResponseBody}};
    use dotenv::dotenv;
    use chrono::Local;
    use crate::Pool;
    use crate::operations;
    use crate::models::*;
    use diesel::{RunQueryDsl, SqliteConnection, prelude::*, r2d2::{ConnectionManager}};
    fn init_test() -> Pool {
        use crate::schema::user::dsl::*;
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL")
            .expect("Unable to locate the database.\nTry setting the 'DATABASE_URL' variable.");
        let database = Pool::builder()
            .build(ConnectionManager::<SqliteConnection>::new(database_url))
            .expect("Unable to open the database.");
        let db_connection = database.get().unwrap();
        //let _ = diesel::delete(user).execute(&db_connection); //显式忽略其结果，能做就做，做不了就算了
        //let _ = diesel::delete(crate::schema::message::dsl::message).execute(&db_connection);
        let alice = PostUser {
            id: 1,
            name: String::from("Alice"),
            register_date: Local::now().naive_local(),
        };
        let bob = PostUser {
            id: 2,
            name: String::from("Bob"),
            register_date: Local::now().naive_local(),
        };
        let _ = diesel::insert_into(user)
            .values(&alice)
            .execute(&db_connection);
        let _ = diesel::insert_into(user)
            .values(&bob)
            .execute(&db_connection);

        let hi = PostMessage {
            id: 1,
            user: user.filter(name.eq("Alice")).first::<PostUser>(&db_connection).unwrap().id,
            title: String::from("Hi"),
            content: String::from("Hello, world!"),
            pub_date: Local::now().naive_local(),
        };
        let this_is_a_title = PostMessage {
            id: 2,
            user: user.filter(name.eq("Bob")).first::<PostUser>(&db_connection).unwrap().id,
            title: String::from("This is a title"),
            content: String::from("This is my content"),
            pub_date: Local::now().naive_local(),
        };
        let _ = diesel::insert_into(crate::schema::message::dsl::message)
            .values(&hi)
            .execute(&db_connection);
        let _ = diesel::insert_into(crate::schema::message::dsl::message)
            .values(&this_is_a_title)
            .execute(&db_connection);
        database
    }

    #[actix_rt::test]
    async fn test_can_reach() {
        let database = init_test();
        let mut app = test::init_service(
            App::new()
            .data(database.clone())
            .service(operations::get_message)
        ).await;
        let req = test::TestRequest::get().uri("/api/message").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
    }
    #[actix_rt::test]
    async fn test_message_can_be_fetched() {
        use serde_json::json;
        let database = init_test();
        let db_connection = database.get().unwrap();
        let mut app = test::init_service(
            App::new()
            .data(database.clone())
            .service(operations::get_message)
        ).await;
        let req = test::TestRequest::get().uri("/api/message").to_request();
        let mut resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
        let offset = 0;
        let limit = 100;
        let expected_result: Vec<String> = crate::schema::message::dsl::message
            .order(crate::schema::message::dsl::id)
            .limit(limit as i64)
            .offset(offset as i64)
            .load::<PostMessage>(&db_connection)
            .unwrap()
            .into_iter()
            .map(|x| MessageJson::from(x))
            .map(|x| json!(x).to_string())
            .collect();
        let expected_result = json!(expected_result).to_string();
        let actual_result = match resp.take_body() {
            ResponseBody::Body(b) => {
                if let Body::Bytes(bytes) = b {
                    bytes
                } else {
                    Bytes::from("NO")
                }
            },
            ResponseBody::Other(body) => {
                if let Body::Bytes(bytes) = body {
                    bytes
                } else {
                    Bytes::from("NO")
                }
            }
        };
        assert_eq!(actual_result, expected_result);
    }
}