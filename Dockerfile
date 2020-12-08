# build from base rust service image
FROM cosmeng/rust:1.48

# prebuild stuff
WORKDIR /app

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN mkdir src
RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs

# this build is here to help prevent full rebuilds when not needed
# it is based on just the Cargo.toml
RUN cargo build

# now build with the actual source files
COPY . .
ENV PROTOC=/usr/bin/protoc
RUN cargo build

CMD ["./watch.sh"]