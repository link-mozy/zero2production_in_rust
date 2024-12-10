use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
// `String`과 `&str`에 `graphemes` 메서드를 제공하기 위한 확장 트레이드

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub fn parse_subscriber(form: FormData) -> Result<NewSubscriber, String> {
    let name = SubscriberName::parse(form.name)?;
    let email = SubscriberEmail::parse(form.email)?;
    Ok(NewSubscriber { email, name })
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { email, name })
    }
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
    // let new_subscriber = match parse_subscriber(form.0) {
    //     Ok(subscriber) => subscriber,
    //     Err(_) => return HttpResponse::BadRequest().finish(),
    // };
    // let new_subscriber = match NewSubscriber::try_from(form.0) { ... }
    let new_subscriber = match form.0.try_into() {
        Ok(form) => form,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    match insert_subscriber(&pool, &new_subscriber).await {
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
        new_subscripber.email.as_ref(),
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