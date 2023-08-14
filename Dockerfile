FROM ekidd/rust-musl-builder:latest as builder

RUN USER=root cargo new curver-backend
WORKDIR ./curver-backend
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release --locked
RUN rm src/*.rs

ADD . ./

RUN rm ./target/x86_64-unknown-linux-musl/release/deps/curver-backend*
RUN cargo build --release --locked


FROM alpine:latest

ARG APP=/usr/src/app

EXPOSE 8080

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN addgroup -S $APP_USER \
    && adduser -S -g $APP_USER $APP_USER

RUN apk update \
    && apk add --no-cache ca-certificates tzdata \
    && rm -rf /var/cache/apk/*

COPY --from=builder /home/rust/src/curver-backend/target/x86_64-unknown-linux-musl/release/curver-backend ${APP}/curver-backend

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./curver-backend"]
