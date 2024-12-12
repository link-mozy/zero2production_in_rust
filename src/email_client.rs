use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use crate::domain::SubscriberEmail;

#[derive(Clone)]
pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
    // 우발적인 로깅을 원하지 않는다.
    authorization_token: Secret<String>,
}

impl EmailClient {
    pub fn new(
        base_url: String, 
        sender: SubscriberEmail, 
        authorization_token: Secret<String>,
        timeout: std::time::Duration,
    ) -> Self {
        let http_client = Client::builder()
            .timeout(timeout)
            .build()
            .unwrap();
        Self {
            http_client,
            base_url,
            sender,
            authorization_token,
        }
    }
    
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str
    ) -> Result<(), reqwest::Error> {
        // TODO. `base_url`의 타입을 `String`에서 `request::Url`로 변경하면,
        // `request::Url::join`을 사용할 수 있다.
        let url = format!("{}/email", self.base_url);
        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body: html_content,
            text_body: text_content,
        };
        self
            .http_client
            .post(&url)
            .header(
                "X-Postmark-Server-Token",
                self.authorization_token.expose_secret()
            )
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
// lifetime parameter는 항상 아포스트로피(`'`)로 시작한다.
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use wiremock::Request;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::{header, header_exists, method, path};
    use wiremock::matchers::any;
    use claim::{assert_err, assert_ok};
    use secrecy::Secret;

    /// 무작위로 이메일 제목을 생성한다.
    fn subject() -> String {
        Sentence(1..2).fake()
    }

    /// 무작위로 이메일 내용을 생성한다.
    fn content() -> String {
        Paragraph(1..10).fake()
    }

    /// 무작위로 구독자 이메일을 생성한다.
    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    /// `EmailClient`의 테스트 인스턴스를 얻는다.
    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url,
             email(),
              Secret::new(Faker.fake()),
              // 10초보다 훨씬 짧다.
              std::time::Duration::from_millis(200),
            )
    }

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            // body를 JSON 값으로 파싱한다.
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);

            if let Ok(body) = result {
                dbg!(&body); // 디버깅 print를 위해 추가함.
                // 필드값을 검사하지 않고, 모든 필수 필드들이 존재하는지 확인한다.
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                // 파싱이 실패하면, 요청을 매칭하지 않는다.
                false
            }
        }
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_return_500() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500)) // 더 이상 200이 아니다.
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // Assert
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_succeeds_if_the_server_return_200() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        // 다른 테스트에 있는 모든 매처를 복사하지 않는다.
        // 이 테스트 목적은 밖으로 보내는 요청에 대한 어서션을 하지 않는 것이다.
        // `send_email`에서 테스트 하기 위한 경로를 트리거하기 위한 최소한의 것만 추가한다.
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // Assert
        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher) // 커스텀 matcher를 사용한다.
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let _ = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // Assert
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        let response = ResponseTemplate::new(200)
            // 3분
            .set_delay(std::time::Duration::from_secs(180));

        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // Assert
        assert_err!(outcome);
    }
}