use borsh::{BorshDeserialize, BorshSerialize};
use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use slice_rbtree::{tree_size, RBTree};
use std::collections::BTreeMap;

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone, PartialEq)]
struct MyType {
    array: [u8; 10],
    float: f64,
    num: u64,
    num2: u32,
}

pub fn btreemap(c: &mut Criterion) {
    let map = (0..100)
        .map(|i| {
            let i6 = i as u64;
            let i3 = i as u32;
            let value = MyType {
                array: [i; 10],
                float: f64::from(i) * 1.25,
                num: i6 * i6 * i6 + i6 * i6 + i6,
                num2: 5 * i3 + 8 * i3 * i3,
            };
            (i, value)
        })
        .collect::<BTreeMap<_, _>>();

    let mut map_buffer = vec![0u8; 50_0];

    map.serialize(&mut map_buffer).unwrap();

    let expected_value = MyType {
        array: [3; 10],
        float: 3.0 * 1.25,
        num: 27 + 9 + 3,
        num2: 5 * 3 + 8 * 9,
    };

    c.bench_function("BTreeMap", |b| {
        b.iter(|| {
            let map =
                BTreeMap::<u8, MyType>::deserialize(&mut black_box(map_buffer.as_slice())).unwrap();
            //dbg!(&map);
            assert_eq!(map.get(&3), Some(&expected_value));
        })
    });
}

criterion_group!(benches, btreemap);
criterion_main!(benches);
