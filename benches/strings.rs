use std::string::String;
use std::time::Duration;

use arrayvec::ArrayString;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::measurement::WallTime;
use criterion::Bencher;
use criterion::BenchmarkGroup;
use criterion::BenchmarkId;
use criterion::Criterion;
use imstr::ImString;
use inlinable_string::InlinableString;
use inlinable_string::InlineString;
use smol_str::SmolStr;

use rocstr::RocStr;

const TIME: u64 = 100;

fn generic_bench_clone<T>(group: &mut BenchmarkGroup<WallTime>, param: &'static str)
where
    T: TryFrom<&'static str> + Clone + EqStr + Name,
{
    if let Ok(p) = T::try_from(param) {
        if p.eq(param) {
            group.bench_with_input(
                BenchmarkId::new(T::name(), param.len()),
                &p,
                |b: &mut Bencher<WallTime>, p: &T| b.iter(|| p.clone()),
            );
        }
    }
}

fn bench_clones(c: &mut Criterion) {
    let params = [
        (""),
        ("ab"),
        ("abcd"),
        ("abcdefgh"),
        (core::str::from_utf8(&[b'a'; 16]).unwrap()),
        (core::str::from_utf8(&[b'b'; 32]).unwrap()),
        (core::str::from_utf8(&[b'c'; 64]).unwrap()),
        (core::str::from_utf8(&[b'd'; 128]).unwrap()),
        (core::str::from_utf8(&[b'e'; 256]).unwrap()),
    ];
    let mut group = c.benchmark_group("clone");
    group.measurement_time(Duration::from_millis(TIME));
    group.warm_up_time(Duration::from_millis(TIME));

    for (i, param) in params.iter().enumerate() {
        generic_bench_clone::<String>(&mut group, param);
        generic_bench_clone::<ImString>(&mut group, param);
        generic_bench_clone::<InlinableString>(&mut group, param);
        generic_bench_clone::<SmolStr>(&mut group, param);

        match i {
            4 => {
                generic_bench_clone::<InlineString>(&mut group, param);
                generic_bench_clone::<ArrayString<16>>(&mut group, param);
                generic_bench_clone::<RocStr<16>>(&mut group, param);
            }
            5 => {
                generic_bench_clone::<ArrayString<32>>(&mut group, param);
                generic_bench_clone::<RocStr<32>>(&mut group, param);
            }
            6 => {
                generic_bench_clone::<ArrayString<64>>(&mut group, param);
                generic_bench_clone::<RocStr<64>>(&mut group, param);
            }
            7 => {
                generic_bench_clone::<ArrayString<128>>(&mut group, param);
                generic_bench_clone::<RocStr<128>>(&mut group, param);
            }
            8 => {
                generic_bench_clone::<ArrayString<256>>(&mut group, param);
                generic_bench_clone::<RocStr<256>>(&mut group, param);
            }
            _ => {
                generic_bench_clone::<InlineString>(&mut group, param);
                generic_bench_clone::<ArrayString<8>>(&mut group, param);
                generic_bench_clone::<RocStr<8>>(&mut group, param);
            }
        }
    }

    group.finish();
}

criterion_group!(strings, bench_clones);
criterion_main!(strings);

trait Name {
    fn name() -> &'static str;
}

impl Name for String {
    #[inline]
    fn name() -> &'static str {
        "String"
    }
}

impl Name for ImString {
    #[inline]
    fn name() -> &'static str {
        "ImString"
    }
}

impl Name for InlineString {
    #[inline]
    fn name() -> &'static str {
        "InlineString"
    }
}
impl Name for InlinableString {
    #[inline]
    fn name() -> &'static str {
        "InlinableString"
    }
}
impl Name for SmallString {
    #[inline]
    fn name() -> &'static str {
        "SmallString"
    }
}
impl Name for ArrayString<8> {
    #[inline]
    fn name() -> &'static str {
        "ArrayString"
    }
}
impl Name for ArrayString<16> {
    #[inline]
    fn name() -> &'static str {
        "ArrayString"
    }
}
impl Name for ArrayString<32> {
    #[inline]
    fn name() -> &'static str {
        "ArrayString"
    }
}
impl Name for ArrayString<64> {
    #[inline]
    fn name() -> &'static str {
        "ArrayString"
    }
}
impl Name for ArrayString<128> {
    #[inline]
    fn name() -> &'static str {
        "ArrayString"
    }
}
impl Name for ArrayString<256> {
    #[inline]
    fn name() -> &'static str {
        "ArrayString"
    }
}
impl Name for SmolStr {
    #[inline]
    fn name() -> &'static str {
        "SmolStr"
    }
}

impl Name for RocStr<8> {
    #[inline]
    fn name() -> &'static str {
        "RocStr"
    }
}
impl Name for RocStr<16> {
    #[inline]
    fn name() -> &'static str {
        "RocStr"
    }
}
impl Name for RocStr<32> {
    #[inline]
    fn name() -> &'static str {
        "RocStr"
    }
}
impl Name for RocStr<64> {
    #[inline]
    fn name() -> &'static str {
        "RocStr"
    }
}
impl Name for RocStr<128> {
    #[inline]
    fn name() -> &'static str {
        "RocStr"
    }
}
impl Name for RocStr<256> {
    #[inline]
    fn name() -> &'static str {
        "RocStr"
    }
}

trait EqStr {
    fn eq(&self, rhs: &str) -> bool;
}

impl EqStr for String {
    #[inline]
    fn eq(&self, rhs: &str) -> bool {
        self == rhs
    }
}
impl EqStr for ImString {
    #[inline]
    fn eq(&self, rhs: &str) -> bool {
        self == rhs
    }
}
impl EqStr for InlineString {
    #[inline]
    fn eq(&self, rhs: &str) -> bool {
        self == rhs
    }
}
impl EqStr for InlinableString {
    #[inline]
    fn eq(&self, rhs: &str) -> bool {
        self == rhs
    }
}
impl EqStr for SmallString {
    #[inline]
    fn eq(&self, rhs: &str) -> bool {
        &rhs == self
    }
}
impl EqStr for SmolStr {
    #[inline]
    fn eq(&self, rhs: &str) -> bool {
        self == rhs
    }
}
impl<const SIZE: usize> EqStr for ArrayString<SIZE> {
    #[inline]
    fn eq(&self, rhs: &str) -> bool {
        self == rhs
    }
}
impl<const SIZE: usize> EqStr for RocStr<SIZE> {
    #[inline]
    fn eq(&self, rhs: &str) -> bool {
        self == rhs
    }
}
