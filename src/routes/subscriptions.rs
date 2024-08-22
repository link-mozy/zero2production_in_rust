use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
// `String`과 `&str`에 `graphemes` 메서드를 제공하기 위한 확장 트레이드
use unicode_segmentation::UnicodeSegmentation;

use crate::domain::{NewSubscriber, SubscriberName};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>, 
    pool: web::Data<PgPool>,
) -> HttpResponse {
    // 'web::Form'은 'FormData'의 래퍼다.
    // 'form.0'을 사용하면 기반 'FormData'에 접근할 수 있다.
    let new_subscripber = NewSubscriber {
        email: form.0.email,
        name: SubscriberName::parse(form.0.name),
    };
    // if !is_valid_name(&form.name) {
    //     return HttpResponse::BadRequest().finish();
    // }
    match insert_subscriber(&pool, &new_subscripber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
// subscribe: 필요한 루틴을 호출해서 처리할 일을 조율하고, HTTP 프로토콜의 규칙과 관습에 따라
// 그 결과물을 적절한 응답으로 변환한다.

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscripber, pool),
)]
pub async fn insert_subscriber(
    pool: &PgPool, 
    new_subscripber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        new_subscripber.email,
        new_subscripber.name.inner_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
// insert_subscriber: 데이터베이스 로직을 처리하고, 둘러싸고 있는 웹 프레임워크에는 신경 쓰지 않는다.
// 즉, 입력 타입으로 web::Form이나 web::Data 래퍼를 전달하지 않는다.

/// 입력이 subscriber 이름에 대한 검증 제약 사항을 모두 만족하면 'true'를 반환한다.
/// 그렇지 않으면 'false'를 반환한다.
pub fn is_valid_name(s: &str) -> bool {
    let is_empty_or_whitespace = s.trim().is_empty();
    let is_too_long = s.graphemes(true).count() > 256;
    let forbidden_charactes = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
    let contains_forbidden_characters = s
        .chars()
        .any(|g| forbidden_charactes.contains(&g));

    !(is_empty_or_whitespace || is_too_long || contains_forbidden_characters)
}