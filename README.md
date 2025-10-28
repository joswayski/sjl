# sjl - Simple JSON Logger
⚠️ WIP ⚠️




### Why?
I mostly need JSON logging without the quirks: [enums that serialize correctly](https://github.com/tokio-rs/tracing/issues/3051) and [clean output out of the box](https://josevalerio.com/rust-json-logging), *not* escaped strings.

I built this because the [tracing crate](https://crates.io/crates/tracing)'s `valuable` support has been behind an [unstable feature flag for over three years](https://github.com/tokio-rs/tracing/discussions/1906) and the  [slog](https://crates.io/crates/slog) crate also doesn't seem to provide this..

If you want a simple JSON logger, this might be useful for you too.



