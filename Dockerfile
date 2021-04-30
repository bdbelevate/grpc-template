# build from base rust service image
FROM cosmeng/rust-service-base:v2.2 as builder

WORKDIR /app

# # now build with the actual source files
COPY src src
COPY proto proto
COPY build.rs build.rs

RUN touch build.rs
RUN touch src/main.rs

ENV PROTOC=/usr/bin/protoc
RUN cargo build

FROM cosmeng/alpine-run-base:1.0

# add minimum requirements
RUN apk update &&\
    apk add binutils musl protoc openssl-dev

COPY --from=builder /app/target/debug/rust-service .

# run the service
CMD ["./rust-service"]