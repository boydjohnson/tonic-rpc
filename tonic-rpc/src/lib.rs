//! `tonic-rpc` is a macro that generates the traits and stubs used by [`tonic`](https://crates.io/crates/tonic)
//! from Rust definitions instead of `proto` files.
//!
//! This means that you can get all the [benefits](https://github.com/hyperium/tonic#features)
//! of `tonic` while using regular Rust types and without needing to use `proto` files or build scripts.
//! Of course, this comes at the sacrifice of interoporability.
//!
//! # Alternatives
//! [`tarpc`](https://crates.io/crates/tarpc) is an excellent RPC library that also defines services using
//! as a Rust trait.
//!
//! # Required dependencies
//! ```toml
//! tonic = "0.4"
//! tonic-rpc = { version = "0.1", features = [ <enabled-codecs> ] }
//! ```
//!
//! # Example
//! Instead of defining a `proto`, define a service as a trait:
//! ```no_run
//! # #[cfg(feature = "json")]
//! #[tonic_rpc::tonic_rpc(json)]
//! trait Increment {
//!     fn increment(arg: i32) -> i32;
//! }
//! # fn main() {}
//! ```
//! The attribute **`#[tonic_rpc(json)]`** indicates that this service
//! will serialize the requests and responses using `json`.
//! Other [`encodings are available`](#encodings).
//! The arguments and return values for each function must implement
//! `serde::Serialize` and `serde::Deserialize`.
//!
//! The service can be implemented by defining and `impl`:
//! ```no_run
//! # #[cfg(feature = "json")]
//! # #[tonic_rpc::tonic_rpc(json)]
//! # trait Increment {
//! #     fn increment(arg: i32) -> i32;
//! # }
//! struct State;
//!
//! # #[cfg(feature = "json")]
//! #[tonic::async_trait]
//! impl increment_server::Increment for State {
//!     async fn increment(
//!         &self,
//!         request: tonic::Request<i32>,
//!     ) -> Result<tonic::Response<i32>, tonic::Status> {
//!         Ok(tonic::Response::new(request.into_inner() + 1))
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! And a server and client can be run:
//! ```rust
//! # #[cfg(feature = "json")]
//! # #[tonic_rpc::tonic_rpc(json)]
//! # trait Increment {
//! #     fn increment(arg: i32) -> i32;
//! # }
//! # struct State;
//! #
//! # #[cfg(feature = "json")]
//! # #[tonic::async_trait]
//! # impl increment_server::Increment for State {
//! #     async fn increment(
//! #         &self,
//! #         request: tonic::Request<i32>,
//! #     ) -> Result<tonic::Response<i32>, tonic::Status> {
//! #       Ok(tonic::Response::new(request.into_inner() + 1))
//! #   }
//! # }
//! # #[cfg(feature = "json")]
//! async fn run_client_server() {
//!     let mut listener = tokio::net::TcpListener::bind("[::1]:8080").await.unwrap();
//!     let addr = listener.local_addr().unwrap();
//!     tokio::spawn(async move {
//!         tonic::transport::Server::builder()
//!             .add_service(increment_server::IncrementServer::new(State))
//!             .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
//!             .await
//!     });
//!     let mut client = increment_client::IncrementClient::connect(format!("http://{}", addr))
//!         .await
//!         .unwrap();
//!     let response = client.increment(32).await.unwrap().into_inner();
//!     assert_eq!(33, response);
//! }
//! # #[cfg(feature = "json")]
//! # fn main() {
//! #     run_client_server();
//! # }
//! # #[cfg(not(feature = "json"))]
//! # fn main() {}
//! ```
//!
//! The full example is available [here](https://github.com/adamrk/tonic-rpc/tree/main/example).
//! Further examples are available in the [tests folder](https://github.com/adamrk/tonic-rpc/tree/main/tonic-rpc/tests).
//!
//! # Encodings
//! Multiple codecs are available for serializing the RPC request/response types.
//! Each codec is enabled by a [feature flag](https://doc.rust-lang.org/cargo/reference/features.html#the-features-section).
//! **At least one of these features must be enabled.**
//! - **`bincode`** - using [`bincode`](https://crates.io/crates/bincode)
//! - **`cbor`** - using [`serde_cbor`](https://crates.io/crates/serde_cbor)
//! - **`json`** - using [`serde_json`](https://crates.io/crates/serde_json)
//! - **`messagepack`** - using [`rmp-serde`](https://crates.io/crates/rmp-serde)
//!
//! E.g. To use the encode using `bincode`, use the attribute
//! ```ignore
//! #[tonic_rpc::tonic_rpc(cbor)]
//! ```
//! and include
//! ```toml
//! tonic-rpc = { version = "0.1", features = [ "cbor" ]}
//! ```
//! in `Cargo.toml`.
//!
//! # Streaming
//! Streaming can be added on the client or server side by adding the attributes
//! `#[client_streaming]` or `#[server_streaming]` to a function in the service trait.
//! These behave the same as if the `stream` keyword were added to a `proto` definition.
//!
//! Examples that use streaming can be found in the [tests folder](https://github.com/adamrk/tonic-rpc/tree/main/tonic-rpc/tests).
//!
//! # Request/Response types
//!
//! The traits and functions generated by [`tonic-rpc`] will be transformations
//! of the methods defined in the [`tonic_rpc`] trait to add handling of `gRPC`
//! request/response types, async, and streaming.
//!
//! This is a summary of how signatures are transformed:
//!
//! ## Arguments
//! ```ignore
//! fn f(x: X, y:Y) -> ..
//! ```
//! becomes
//! ```ignore
//! async fn f(&self, arg: tonic::Request<(X,Y)>) -> ..
//! ```
//!
//! ## Return value
//! ```ignore
//! fn f(..) -> Z
//! ```
//! becomes
//! ```ignore
//! async fn f(..) -> Result<tonic::Response<Z>, tonic::Status>
//! ```
//!
//! ## Streaming arguments
//! ```ignore
//! #[client_streaming]
//! fn f(x:X) -> ..
//! ```
//! becomes
//! ```ignore
//! async fn f(&self, arg: tonic::Request<tonic::Streaming<X>>) -> ..
//! ```
//!
//! ## Streaming return value
//! ```ignore
//! #[server_streaming]
//! fn f(..) -> Z
//! ```
//! becomes
//! ```ignore
//! type FStream: Stream<Item = Result<Z, tonic::Status>;
//!
//! async fn f(..) -> Result::<tonic::Response<Self::FStream>, tonic::Status>
//! ```
//!

#![cfg_attr(docsrs, feature(doc_cfg))]

pub use tonic_rpc_macro::tonic_rpc;

pub mod codec;
