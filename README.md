# Sample Service

## Note

This is a project template using [cargo generate](https://github.com/ashleygwilliams/cargo-generate). The idea is to create a new service from scratch using this as a template. This includes a sample model and embedded object model as well as a bunch of tests. It requires using the latest version of `cargo generate`. So to install that do:

```sh
cargo install --git https://github.com/ashleygwilliams/cargo-generate
```

Then to create a new project:

```sh
cargo generate --git https://github.com/briandeboer/mongodb-service-template
```

Answer the prompts and voila! you have a new project.

Note: It will rename things and remove the word "service". So if you want to create an Events service with types called Event. Then enter the name as "EventService". That will result in a crate named `event-service` and it will have a type of message called Event.

## Usage

Run the cargo generate above. Then go and edit (and/or rename) the proto/sample.proto. Keep in mind that if you rename it you will need to change the build.rs accordingly.

If you add fields, you'll likely need to change the api/items.rs to update the update_one command. You will also likely want to remove or edit the src/data.rs.

## Getting Started

- Install [Rust](https://www.rust-lang.org/tools/install)
- Run `cargo run` to build and run service

## Install Garden
See garden.io

## VSCode

### Plugins

- Better TOML
- Native Debug
- Rust
- rust-analyzer

## Bloom RPC

Testing the gRPC calls is best done via (BloomRPC)[https://github.com/uw-labs/bloomrpc]. More info on their site but easiest way to install it is via brew.

```bash
brew cask install bloomrpc
```

Once it's installed you can open the proto file to test the endpoints.
