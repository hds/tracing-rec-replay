use std::fmt;

use tracing_subscriber::prelude::*;

#[derive(Debug)]
struct MyValue {
    val: u64,
}

impl fmt::Display for MyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "MyValue has val={val}", val = self.val)
    }
}

fn main() {
    tracing_subscriber::registry()
        .with(tracing_rec::rec_layer())
        .init();

    let my_val = MyValue { val: 42 };
    tracing::info!(field = ?my_val, "debug");
    tracing::info!(field = %my_val, "display");

    tracing::info!(field = 12.3_f64, "f64");
    tracing::info!(field = -12_i64, "i64");
    tracing::info!(field = 12_u64, "u64");
    tracing::info!(field = -12_i128, "i128");
    tracing::info!(field = 12_u128, "u128");
    tracing::info!(field = true, "bool");
    tracing::info!(field = "hello", "str");

    tracing::info_span!(
        "fields",
        field_debug = ?my_val,
        field_display = %my_val,
        field_f64 = 12.3_f64,
        field_i64 = -12_i64,
        field_u64 = 12_u64,
        field_i128 = -12_i128,
        field_u128 = 12_u128,
        field_bool = true,
        field_str = "hello",
    );

    let span = tracing::info_span!(
        "fields",
        field_debug = tracing::field::Empty,
        field_display = tracing::field::Empty,
        field_f64 = tracing::field::Empty,
        field_i64 = tracing::field::Empty,
        field_u64 = tracing::field::Empty,
        field_i128 = tracing::field::Empty,
        field_u128 = tracing::field::Empty,
        field_bool = tracing::field::Empty,
        field_str = tracing::field::Empty,
    );

    span.record("field_debug", tracing::field::debug(&my_val));
    span.record("field_display", tracing::field::display(&my_val));
    span.record("field_f64", 12.3_f64);
    span.record("field_i64", -12_i64);
    span.record("field_u64", 12_u64);
    span.record("field_i128", -12_i128);
    span.record("field_u128", 12_u128);
    span.record("field_bool", true);
    span.record("field_str", "hello");
}
