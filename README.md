<div align="center">

## SLAS
*Static Linear Algebra System*

[![Crates.io](https://img.shields.io/crates/v/slas?logo=rust&style=flat-square)](https://crates.io/crates/slas)
[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/unic0rn9k/slas/Tests?label=tests&logo=github&style=flat-square)](https://github.com/unic0rn9k/slas/actions/workflows/rust.yml)
[![Docs](https://img.shields.io/docsrs/slas/latest?logo=rust&style=flat-square)](https://docs.rs/slas/latest/slas/)
[![Donate on paypal](https://img.shields.io/badge/paypal-donate-1?style=flat-square&logo=paypal&color=blue)](https://www.paypal.com/paypalme/unic0rn9k/5usd)

</div>

Provides statically allocated vector, matrix and tensor types, for interfacing with blas/blis, in a performant manner, using copy-on-write (aka cow) behavior by default.

[What is BLAS?](http://www.netlib.org/blas/)

### Example

```rust
use slas::prelude::*;
let a = moo![f32: 1, 2, 3.2];
let b = moo![f32: 3, 0.4, 5];
println!("Dot product of {a:?} and {b:?} is {:?}", a.dot(&b));
```
You can also choose a static backend yourself
```rust
use slas::prelude::*;
let a = moo![on slas_backend::Rust:f32: 1, 2, 3.2];
// This will only use rust code for all operations on a
```

```rust
use slas::prelude::*;
let a = moo![on slas_backend::Blas:f32: 1, 2, 3.2];
// This will always use blas for all operations on a
```

The `StaticCowVec` derefences to `StaticVecUnino`, which in turn dereferences to `[T; LEN]`,
so any method implemented for `[T;LEN]` can also be called on `StaticCowVec` and `StaticVecUnion`.

[More example code here.](https://github.com/unic0rn9k/slas/blob/master/tests/src/main.rs)

### What is a cow and when is it usefull?
The copy-on-write functionality is inspired by [std::borrow::cow](https://doc.rust-lang.org/std/borrow/enum.Cow.html).
The idea is simply that allocations (and time) can be saved, by figuring out when to copy at runtime instead of at compiletime.
This can be memory inefficient at times (as an enum takes the size of its largest field + tag size), which is why you can optionally use `StaticVecUnion`s and `StaticVec`s instead.
You can call `moo`, `moo_ref` and `mut_moo_ref` on any type that implements `StaticVec` to cast it to a appropriate type for it's use-case, with zero overhead.

**moo_ref** returns a `StaticVecRef`, which is just a type alias for a reference to a `StaticVecUnion`.
This is most efficient when you know you don't need mutable access or ownership of a vector.

**mut_moo_ref** returns a `MutStaticVecRef`.
This is a lot like `moo_ref`, but is usefull when you want to mutate your data in place (fx if you wan't to normalize a vector).
You should only use this if you want mutable access to a vector WITH sideeffects.

**moo** returns a `StaticCowVec` that references `self`. This is usefull if you don't know if you need mutable access to you vector and you don't want sideeffects.
If you want to copy data into a `StaticCowVec` then `StaticCowVec::from` is what you need.

**moo_owned** will just return a `StaticVecUnion`. This is usefull when you really just wan't a [T; LEN],
but you need methods only implemented for a `StaticVecUnion`.

 ### Example of cow behavior
```rust
use slas::prelude::*;

let source: Vec<f32> = vec![1., 2., 3.];
let mut v = source.moo();

// Here we mutate v,
// so the content of source will be copied into v before the mutation occours.
v[0] = 0.;

assert_eq!(**v, [0., 2., 3.]);
assert_eq!(source, vec![1., 2., 3.]);
```

The borrow checker won't allow mutating `source` after `v` is created, because assignment to borrowed values is not allowed.
This can be a problem in some situations.

```rust
use slas::prelude::*;

let mut source: Vec<f32> = vec![1., 2., 3.];
let mut v = unsafe { StaticCowVec::<f32, 3>::from_ptr(source.as_ptr()) };

// Here we can mutate source, because v was created from a raw pointer.
source[1] = 3.;
v[0] = 0.;
source[2] = 4.;

assert_eq!(**v, [0., 3., 3.]);
assert_eq!(source, vec![1., 3., 4.]);
```
In the example above, you can see `v` changed value the first time `source` was mutated, but not the second time.
This is because `v` was copied when it was mutated.

### Matrix example

```rust
use slas::prelude::*;
use slas_backend::*;

let a = moo![f32: 1..=6].matrix::<Blas, 2, 3>();
let b = moo![f32: 1..=6].matrix::<Blas, 3, 2>();
let c = a.matrix_mul(&b);

assert_eq!(c, [22., 28., 49., 64.]);

println!("{a:.0?} * {b:.0?} = {:.0?}", c.matrix::<Blas, 2, 2>());
```

Indexing into matricies can be done both with columns and rows first.
When indexing with `[usize; 2]` it will take columns first, where as using `m!` will be rows first.

```rust
use slas::prelude::*;
use slas_backend::*;

let a = moo![f32: 1..=6].matrix::<Blas, 2, 3>();

assert_eq!(a[[0, 1]], a[m![1, 0]]);
```

### Tensor example
At the moment tensors can't do much
```rust
use slas::prelude::*;
let t = moo![f32: 0..27].reshape(&[3, 3, 3], slas_backend::Rust);
assert_eq!(t[[0, 0, 1]], 9.);
```
Thats pretty much it for now...

### Why not just use ndarray (or alike)?
Slas can be faster than ndarray in some specific use cases, like when having to do a lot of allocations, or when using referenced data in vector operations.
Besides slas should always be atleast as fast as ndarray, so it can't hurt.

Ndarray will always use the backend you choose in your cargo.toml.
With slas you can choose a backend in code and even create your own backend that fits your needs.

Statical allocation and the way slas cow behavior works with the borrow checker,
also means that you might catch a lot of bugs at compiletime,
where ndarray most of the time will let you get away with pretty much anything.
For example taking the dot product of two vectors with different sizes,
will cause a panic in ndarray and a compiletime error in slas.

### Installation
By default slas will assume you have blis installed on your system.
If you want tos choose your own blas provider please set `dependencies.slas.default-features = false` in your `Cargo.toml`,
and refer to [blas-src](https://lib.rs/crates/blas-src) for further instructions.
Remember to add `extern crate blas_src;` if you use blas-src as a blas provider.

On the crates.io version of slas (v0.1.0 and 0.1.1) blis is compiled automatically.

For now, if you want to use the newest version of slas, you need to install blis/blas on your system.
- On Arch linux [blis-cblas](https://aur.archlinux.org/packages/blis-cblas/) v0.7.0 from the AUR has been tested and works fine.
- On Debian you can simply run `apt install libblis-dev`.
- On Windows [openblas-src](https://github.com/blas-lapack-rs/openblas-src) has been tested.
This mean you will need to disable slas default features,
follow the installation instructions in the openblas readme and add `extern crate openblas_src` to your main file.

### Misc
- Slas is still in very early days, and is subject to a lot of breaking changes.
- [Benchmarks, tests and related](https://github.com/unic0rn9k/slas/tree/master/tests)

### TODO
- Matrix multiplication operation should use a buffer and `*` operator should return matrix with correct shape
- ~~Rust version of blas functions allowing for loop unrolling - also compile time checks for choosing fastest function~~
- Feature support for conversion between [ndarray](lib.rs/ndarray) types
- Allow for use on stable channel - perhabs with a stable feature
- Implement stable tensors - perhabs for predefined dimensions with a macro
- ~~Make StaticCowVec backed by a union - so that vectors that are always owned can also be supported (useful for memory critical systems, fx. embeded devices).~~
- ~~Modular backends - [like in coaster](https://github.com/spearow/juice/tree/master/coaster)~~
    - GPU support - maybe with cublas
    - ~~Pure rust support - usefull for irust and jupyter support.~~
    - `DynacmicBackend` for selecting backends at runtime
- ~~Refactor backends to make it more generic~~
    - Default backend for default operations

### TODO Before v0.2.0 🎉
- ~~Feature flag for choosing own blas provider~~
- ~~More operations implemented for backends~~
- ~~Rewrite documentation~~
- ~~`WithStaticBackend` struct for vectors with associated backends~~
- ~~Make less terrible benchmarks~~
- ~~`Normalize` operation for backends - to prove mutable access to vectors also work in backends, even with StaticCowVecs.~~

License: Apache-2.0
