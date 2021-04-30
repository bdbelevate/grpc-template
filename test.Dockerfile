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

# Run tests
RUN cargo test