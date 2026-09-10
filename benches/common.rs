#[macro_export]
macro_rules! bench_encode_and_decode {
    ($benches:ident | $ty:ty , encode { $( $encode_name:ident : $values_to_encode:expr ),+ $(,)? } $(,)? decode { $( $decode_name:ident : $values_to_decode:expr ),+ $(,)? } $(,)? ) => {
        $(
            fn $encode_name (c: &mut criterion::Criterion) {
                let value: $ty = $values_to_encode;
                c.bench_function(stringify!($encode_name), |b| b.iter(|| dfu_codecs::codec::Encode::encode_start(&value, &dfu_codecs::JsonOps)));
                c.bench_function(concat!(stringify!($encode_name), "_serde"), |b| b.iter(|| serde_json::to_value(&value)));
            }
        )+

        $(
            fn $decode_name (c: &mut criterion::Criterion) {
                let value: serde_json::Value = $values_to_decode;
                general_bench_decode::<$ty>(c, stringify!($decode_name), concat!(stringify!($decode_name), "_serde"), value);
            }
        )+

        criterion_group!(
            $benches,
            $( $encode_name ),+ ,
            $( $decode_name ),+
        );
    };
}

#[macro_export]
macro_rules! common_bench_functions {
    () => {
        fn general_bench_decode<T: for<'de> serde::Deserialize<'de> + dfu_codecs::codec::Decode>(c: &mut criterion::Criterion, normal: &str, serde: &str, value: serde_json::Value) {
            c.bench_function(normal, |b| {
                b.iter_batched(
                    || (),
                    |()| <T>::decode(&dfu_codecs::JsonOps, &value),
                    criterion::BatchSize::LargeInput,
                )
            });
            c.bench_function(serde, |b| {
                b.iter_batched(
                    || value.clone(),
                    serde_json::from_value::<T>,
                    criterion::BatchSize::LargeInput,
                );
            });
        }
    };
}