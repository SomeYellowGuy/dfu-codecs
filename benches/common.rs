#[macro_export]
macro_rules! bench_encode_and_decode_with_serde {
    ($benches:ident , $ty:ty  , encode { $( $encode_name:ident : $values_to_encode:expr ),+ $(,)? } $(,)? decode { $( $decode_name:ident : $values_to_decode:expr ),+ $(,)? } $(,)? ) => {
        $(
            fn $encode_name (c: &mut criterion::Criterion) {
                let mut group = c.benchmark_group(stringify!($encode_name));
                let value: $ty = std::hint::black_box($values_to_encode);
                group.bench_function("dfu", |b| b.iter(|| dfu_codecs::codec::Encode::encode_start(&value, &dfu_codecs::JsonOps)));
                group.bench_function("serde", |b| b.iter(|| serde_json::to_value(&value)));
                group.finish();
            }
        )+

        $(
            fn $decode_name (c: &mut criterion::Criterion) {
                let mut group = c.benchmark_group(stringify!($decode_name));
                let value: serde_json::Value = std::hint::black_box($values_to_decode);
                group.bench_function("dfu", |b| b.iter(|| <$ty as dfu_codecs::codec::Decode>::decode(&dfu_codecs::JsonOps, &value)));
                group.bench_function("serde", |b| b.iter_batched(|| value.clone(), |v| serde_json::from_value::<$ty>(v), criterion::BatchSize::SmallInput));
                group.finish();
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
macro_rules! bench_encode_and_decode {
    ($benches:ident , $ty:ty , encode { $( $encode_name:ident : $values_to_encode:expr ),+ $(,)? } $(,)? decode { $( $decode_name:ident : $values_to_decode:expr ),+ $(,)? } $(,)? ) => {
        $(
            fn $encode_name (c: &mut criterion::Criterion) {
                let value: $ty = $values_to_encode;
                c.bench_function(stringify!($encode_name), |b| b.iter(|| dfu_codecs::codec::Encode::encode_start(&value, &dfu_codecs::JsonOps)));
            }
        )+

        $(
            fn $decode_name (c: &mut criterion::Criterion) {
                let value: serde_json::Value = $values_to_decode;
                c.bench_function(stringify!($decode_name), |b| {
                    b.iter_batched(
                        || (),
                        |()| <$ty as dfu_codecs::codec::Decode>::decode(&dfu_codecs::JsonOps, &value),
                        criterion::BatchSize::LargeInput,
                    )
                });
            }
        )+

        criterion_group!(
            $benches,
            $( $encode_name ),+ ,
            $( $decode_name ),+
        );
    };
}
