FROM rust:1.78.0

# 작업 디렉토리를 `app`으로 변경한다.(`cd app`과 동일하다.)
# `app` 폴더가 존재하지 않는 경우 도커가 해당 폴더를 생성한다.
WORKDIR /app
# 구성을 연결하기 위해 필요한 시스템 디펜던시를 설치한다.
RUN apt update && apt install lld clang -y
# 작업 환경의 모든 파일을 도커 이미지로 복사한다.
COPY . .
# sqlx가 실제 데이터베이스에 쿼리를 시도하는 대신, 지정된 메타데이터를 보게 한다.
ENV SQLX_OFFLINE true
# 바이너리를 빌드하자.
# 빠르게 빌드하기 위해 release 프로파일을 사용한다.
RUN cargo build --release
ENV RUST_BACKTRACE 1
ENV APP_ENVIRONMENT production
# `docker run`이 실행되면, 바이너리를 구동한다.
ENTRYPOINT ["./target/release/zero2prod"]