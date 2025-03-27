use std::future::Future;

use anyhow::{Context, Result};
use msfs::network::NetworkRequestBuilder;
use serde::{de::DeserializeOwned, Deserialize};

use crate::futures::AsyncNetworkRequest;

/// The trait definition for a function that can be called through the navigation data interface
trait Function: DeserializeOwned {
    /// Create a new instance of the function
    ///
    /// * `args` - A `serde_json::Value`` with the data passed in the call
    fn new(args: serde_json::Value) -> Result<Self> {
        let mut instance = serde_json::from_value::<Self>(args)
            .context("can't deserialize self from args: {args}")?;

        instance.init()?;

        Ok(instance)
    }

    /// Any custom initialization logic to call before execution
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    /// The main function entry
    async fn run(&mut self) -> Result<()>;
}

/// The DownloadNavigationData function
///
/// Usage: Download a DFD database from a signed URL
#[derive(Deserialize)]
pub struct DownloadNavigationData {
    url: String,
}

impl Function for DownloadNavigationData {
    async fn run(&mut self) -> Result<()> {
        let data = NetworkRequestBuilder::new(&self.url)
            .context("can't create new NetworkRequestBuilder")?
            .get()
            .context(".get() returned None")?
            .wait_for_data()
            .await?;

        Ok(())
    }
}

/// Generates boilerplate code for wrapping async functions in a uniform interface.
///
/// This macro simplifies the process of exposing a set of structs that implement an async `run` method
/// (via a `Function` trait) into a single deserializable enum for runtime dispatch and execution.
///
/// # Example
///
/// ```rust
/// #[derive(Deserialize)]
/// pub struct Foo {
///     bar: String,
/// }
///
/// impl Function for Foo {
///     async fn run(&mut self) -> Result<()> {
///         // Do some work...
///         Ok(())
///     }
/// }
///
/// define_interface_functions!(Foo);
/// ```
///
/// The macro will generate:
///
/// - A `FooWrapper` struct that owns a future created from the `run` method.
/// - An `InterfaceFunction` enum with a variant for each provided type (e.g. `Foo(FooWrapper)`).
/// - Implementations for `Deserialize`, `run`, and `id` on `InterfaceFunction`.
///
/// # JSON Input Example
///
/// A JSON payload like the following:
///
/// ```json
/// {
///     "id": "1",
///     "function": "Foo",
///     "data": {
///         "bar": "baz"
///     }
/// }
/// ```
///
/// Will deserialize into `InterfaceFunction::Foo(FooWrapper)`, ready to be executed via `.run()`.
///
/// # Execution
///
/// Calling `run()` on an `InterfaceFunction` polls the underlying future once per call,
/// returning either:
/// - `RunStatus::InProgress` if the future isn’t complete yet.
/// - `RunStatus::Finished(result)` if the future resolved.
///
/// This is useful in our environment as we need to yield back to the sim in order not to block the thread, and we may have some functions that aren't able to resolve in a single frame.
///
/// # Note
///
/// During JSON deserialization, the input is validated to ensure the following:
/// - The `id`, `function`, and `data` fields are present.
/// - The `function` field matches the name of a registered function.
/// - The `data` field can be successfully parsed into the corresponding function's expected input type.
macro_rules! define_interface_functions {
    ($($fn_name:ident),* $(,)?) => {
        paste::paste! {
            /// The return status from a call to `run` on a function
            pub enum RunStatus {
                InProgress,
                Finished(anyhow::Result<()>),
            }

            $(
                /// An internal wrapper around a function
                pub struct [<$fn_name Wrapper>] {
                    id: String,
                    future: futures_lite::future::BoxedLocal<anyhow::Result<()>>,
                }

                impl [<$fn_name Wrapper>] {
                    fn new(id: String, args: serde_json::Value) -> anyhow::Result<Self> {
                        let mut instance = $fn_name::new(args)?;
                        // Create the future. Note that this does not start executing until we poll it
                        let future = Box::pin(async move { instance.run().await });
                        Ok(Self { id, future })
                    }

                    fn run(&mut self) -> RunStatus {
                        // We allow the function run to be async in order to wait for certain conditions. However, MSFS WASM modules are not multithreaded so we need to yield back to the main thread.
                        // We get around this by polling once per update, and the continuing to poll (if needed) in later updates.
                        match futures_lite::future::block_on(futures_lite::future::poll_once(&mut self.future)) {
                            Some(result) => {
                                RunStatus::Finished(result)
                            },
                            None => RunStatus::InProgress,
                        }
                    }
                }
            )*

            /// The available functions in the navigation data interface
            pub enum InterfaceFunction {
                $( $fn_name([<$fn_name Wrapper>]), )*
            }

            impl<'de> serde::Deserialize<'de> for InterfaceFunction {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    #[derive(serde::Deserialize)]
                    struct Helper {
                        id: String,
                        function: String,
                        data: serde_json::Value,
                    }

                    let Helper { id, function, data } = Helper::deserialize(deserializer)?;

                    match function.as_str() {
                        $(
                            stringify!($fn_name) => {
                                let wrapper = [<$fn_name Wrapper>]::new(id, data).map_err(serde::de::Error::custom)?;

                                Ok(InterfaceFunction::$fn_name(wrapper))
                            },
                        )*
                        _ => Err(serde::de::Error::custom(format!("Unknown function: {}", function))),
                    }
                }
            }

            impl InterfaceFunction {
                /// Run the function
                pub fn run(&mut self) -> RunStatus {
                    match self {
                        $( Self::$fn_name(wrapper) => wrapper.run(), )*
                    }
                }

                /// Get the unique ID of the function call
                pub fn id(&mut self) -> &str {
                    match self {
                        $( Self::$fn_name(wrapper) => &wrapper.id, )*
                    }
                }
            }
        }
    };
}

define_interface_functions!(DownloadNavigationData);
