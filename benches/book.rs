use codecs::codec::{Decode, Encode, MapDecode, MapEncode, OptionalFieldDecode};
use codecs::{
    DataResult, DynamicOps, JsonOps, MapLike, RecordBuilder, decode_from_map_decode,
    encode_from_map_encode,
};
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize)]
struct Book {
    name: String,
    id: u64,
    pages: Vec<Page>,
    prequel: Option<Box<Book>>,
}

impl MapEncode for Book {
    fn map_encode<O: DynamicOps, B: RecordBuilder<Value = O::Value>>(
        &self,
        ops: &O,
        prefix: B,
    ) -> B {
        prefix
            .add_field(ops, "name", &self.name)
            .add_field(ops, "id", &self.id)
            .add_field(ops, "pages", &self.pages)
            .add_optional_field(ops, "prequel", &self.prequel)
    }
}
encode_from_map_encode!(Book);

impl MapDecode for Book {
    fn map_decode<O: DynamicOps>(
        ops: &O,
        input: &impl MapLike<Value = O::Value>,
    ) -> DataResult<Self> {
        let name = String::decode_field(input, ops, "name");
        let id = u64::decode_field(input, ops, "id");
        let pages = Vec::<Page>::decode_field(input, ops, "pages");
        let prequel = Option::<Box<Book>>::decode_optional_field(input, ops, "prequel", false);
        DataResult::apply_4(
            |name, id, pages, prequel| Book {
                name,
                id,
                pages,
                prequel,
            },
            name,
            id,
            pages,
            prequel,
        )
    }
}
decode_from_map_decode!(Book);

#[derive(Clone, Serialize, Deserialize)]
struct Page {
    content: String,
    index: u32,
    notes: HashMap<String, String>,
}

impl MapEncode for Page {
    fn map_encode<O: DynamicOps, B: RecordBuilder<Value = O::Value>>(
        &self,
        ops: &O,
        prefix: B,
    ) -> B {
        prefix
            .add_field(ops, "content", &self.content)
            .add_field(ops, "index", &self.index)
            .add_field(ops, "notes", &self.notes)
    }
}
encode_from_map_encode!(Page);

impl MapDecode for Page {
    fn map_decode<O: DynamicOps>(
        ops: &O,
        input: &impl MapLike<Value = O::Value>,
    ) -> DataResult<Self> {
        let content = String::decode_field(input, ops, "content");
        let index = u32::decode_field(input, ops, "index");
        let notes = <HashMap<String, String>>::decode_field(input, ops, "notes");
        DataResult::apply_3(
            |content, index, notes| Page {
                content,
                index,
                notes,
            },
            content,
            index,
            notes,
        )
    }
}
decode_from_map_decode!(Page);

fn create_pages() -> Vec<Page> {
    let mut pages = Vec::new();
    for i in 1..=100 {
        let mut notes = HashMap::new();
        notes.insert(String::from("test"), "a".to_string());
        notes.insert(i.to_string(), "b".to_string());
        pages.push(Page {
            index: i,
            content: "Sample page!".to_string(),
            notes,
        })
    }
    pages
}

fn create_book() -> Book {
    Book {
        name: "".to_string(),
        id: 0,
        pages: create_pages(),
        prequel: Some(Box::new(Book {
            name: "Prequel".to_string(),
            id: 981234123,
            pages: create_pages(),
            prequel: None,
        })),
    }
}

fn book_json_with_errors() -> Value {
    let mut pages = create_pages().encode_start(&JsonOps).unwrap();
    let Value::Array(pages_array) = &mut pages else {
        panic!("Expected encoding pages to give a list")
    };
    pages_array.push(json!({"index": 101, "content": "Page 1"}));
    pages_array.push(json!({"index": 102, "content": true}));
    pages_array.push(json!({"index": 103, "content": "Page 3"}));
    pages_array.push(json!(4));
    json!({
        "name": "Error Book",
        "id": 1234,
        "pages": pages,
        "prequel": {}
    })
}

fn bench_encode(c: &mut Criterion) {
    let book = create_book();

    c.bench_function("encode", |b| b.iter(|| book.encode_start(&JsonOps)));
    c.bench_function("encode_serde", |b| b.iter(|| serde_json::to_value(&book)));
}

fn general_bench_decode(c: &mut Criterion, normal: &str, serde: &str, value: Value) {
    c.bench_function(normal, |b| {
        b.iter_batched(
            || (),
            |()| Book::decode(&JsonOps, &value),
            BatchSize::SmallInput,
        )
    });
    c.bench_function(serde, |b| {
        b.iter_batched(
            || value.clone(),
            serde_json::from_value::<Book>,
            BatchSize::SmallInput,
        );
    });
}

fn bench_decode(c: &mut Criterion) {
    let book = create_book();
    let value = book.encode_start(&JsonOps).unwrap();

    general_bench_decode(c, "decode", "decode_serde", value);
}

fn bench_decode_with_errors(c: &mut Criterion) {
    let value = book_json_with_errors();

    general_bench_decode(c, "decode_with_errors", "decode_with_errors_serde", value);
}

criterion_group!(
    benches,
    bench_encode,
    bench_decode,
    bench_decode_with_errors
);
criterion_main!(benches);
