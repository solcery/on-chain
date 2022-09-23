use borsh::{BorshDeserialize, BorshSerialize};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use slice_rbtree::{tree_size, RBTree};
use std::collections::BTreeMap;

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone, Copy, PartialEq)]
struct MyType {
    array: [u8; 10],
    float: f64,
    num: u64,
    num2: u32,
}

impl MyType {
    fn gen(i: u8) -> Self {
        let i6 = u64::from(i);
        let i3 = u32::from(i);
        MyType {
            array: [i; 10],
            float: f64::from(i) * 1.25,
            num: i6 * i6 * i6 + i6 * i6 + i6,
            num2: 5 * i3 + 8 * i3 * i3,
        }
    }
}

pub fn deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("Deserialization");
    let map = (0..100)
        .map(|i| (i, MyType::gen(i)))
        .collect::<BTreeMap<u8, MyType>>();

    let mut map_buffer = vec![0u8; tree_size(1, std::mem::size_of::<MyType>(), 200)];

    map.serialize(&mut map_buffer.as_mut_slice()).unwrap();

    group.bench_function("BTreeMap", |b| {
        b.iter(|| {
            BTreeMap::<u8, MyType>::deserialize(&mut black_box(map_buffer.as_slice())).unwrap()
        })
    });

    let mut slice_map = RBTree::<u8, MyType, 1, { std::mem::size_of::<MyType>() }>::init_slice(
        map_buffer.as_mut_slice(),
    )
    .unwrap();

    for i in 0..100 {
        slice_map.insert(i, MyType::gen(i)).unwrap();
    }

    group.bench_function("RBTree", |b| {
        b.iter(|| unsafe {
            let map = RBTree::<u8, MyType, 1, { std::mem::size_of::<MyType>() }>::from_slice(
                black_box(map_buffer.as_mut_slice()),
            )
            .unwrap();
            assert!(!map.is_empty());
        })
    });
    group.finish();
}

pub fn access_one_value(c: &mut Criterion) {
    let mut group = c.benchmark_group("Access one value");
    let map = (0..100)
        .map(|i| (i, MyType::gen(i)))
        .collect::<BTreeMap<u8, MyType>>();

    let mut map_buffer = vec![0u8; tree_size(1, std::mem::size_of::<MyType>(), 200)];

    map.serialize(&mut map_buffer.as_mut_slice()).unwrap();

    group.bench_function("BTreeMap", |b| {
        let map =
            BTreeMap::<u8, MyType>::deserialize(&mut black_box(map_buffer.as_slice())).unwrap();
        let expected_value = MyType::gen(3);
        b.iter(|| assert_eq!(map.get(&3), Some(&expected_value)))
    });

    let mut slice_map = RBTree::<u8, MyType, 1, { std::mem::size_of::<MyType>() }>::init_slice(
        map_buffer.as_mut_slice(),
    )
    .unwrap();

    for i in 0..100 {
        slice_map.insert(i, MyType::gen(i)).unwrap();
    }

    group.bench_function("RBTree", |b| {
        let map = unsafe {
            RBTree::<u8, MyType, 1, { std::mem::size_of::<MyType>() }>::from_slice(black_box(
                map_buffer.as_mut_slice(),
            ))
            .unwrap()
        };
        let expected_value = MyType::gen(3);
        b.iter(|| assert_eq!(map.get(&3), Some(expected_value)))
    });
    group.finish();
}

pub fn add_one_value(c: &mut Criterion) {
    let mut group = c.benchmark_group("Add one value");
    let map = (0..100)
        .map(|i| {
            let i6 = u64::from(i);
            let i3 = u32::from(i);
            let value = MyType {
                array: [i; 10],
                float: f64::from(i) * 1.25,
                num: i6 * i6 * i6 + i6 * i6 + i6,
                num2: 5 * i3 + 8 * i3 * i3,
            };
            (i, value)
        })
        .collect::<BTreeMap<u8, MyType>>();

    let mut map_buffer = vec![0u8; tree_size(1, std::mem::size_of::<MyType>(), 200)];

    map.serialize(&mut map_buffer.as_mut_slice()).unwrap();

    group.bench_function("BTreeMap", |b| {
        let mut map =
            BTreeMap::<u8, MyType>::deserialize(&mut black_box(map_buffer.as_slice())).unwrap();
        let value = MyType::gen(104);
        b.iter(|| map.insert(104, value))
    });

    let mut slice_map = RBTree::<u8, MyType, 1, { std::mem::size_of::<MyType>() }>::init_slice(
        map_buffer.as_mut_slice(),
    )
    .unwrap();

    for i in 0..100 {
        let i6 = u64::from(i);
        let i3 = u32::from(i);
        let value = MyType {
            array: [i; 10],
            float: f64::from(i) * 1.25,
            num: i6 * i6 * i6 + i6 * i6 + i6,
            num2: 5 * i3 + 8 * i3 * i3,
        };
        slice_map.insert(i, value).unwrap();
    }

    group.bench_function("RBTree", |b| {
        let mut map = unsafe {
            RBTree::<u8, MyType, 1, { std::mem::size_of::<MyType>() }>::from_slice(black_box(
                map_buffer.as_mut_slice(),
            ))
            .unwrap()
        };
        let value = MyType::gen(104);
        b.iter(|| map.insert(104, value))
    });
    group.finish();
}

criterion_group!(benches, deserialization, access_one_value, add_one_value);
criterion_main!(benches);
