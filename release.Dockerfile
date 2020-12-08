# build from standard rust alpine
FROM cosmeng/rust:1.48

# prebuild stuff
WORKDIR /app

COPY Cargo.toml Cargo.toml
RUN mkdir src/
RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs

# this build is here to help prevent full rebuilds when not needed
RUN RUSTFLAGS='-C target-feature=-crt-static' cargo build --release
RUN rm -f target/release/deps/{{project-name}}*

COPY build.rs ./build.rs
COPY ./proto ./proto
ENV PROTOC=/usr/bin/protoc
RUN rm -rf src
COPY ./src ./src
COPY ./examples ./examples
RUN RUSTFLAGS='-C target-feature=-crt-static' cargo build --release


# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM alpine:3.12
RUN apk update &&\
  apk add binutils
# add ssl dependencies
# RUN apk add openssl-dev

RUN addgroup -g 1000 appuser
RUN adduser -D -s /bin/sh -u 1000 -G appuser appuser

WORKDIR /home/{{project-name}}/bin/

RUN mkdir examples
COPY --from=cargo-build /app/examples ./examples
COPY --from=cargo-build /app/target/release/{{project-name}} .

RUN chown appuser:appuser {{project-name}}

USER appuser
CMD ["./{{project-name}}"]