Tools for generating and working with matrices of the forested graph complex.

To generate the `d_u + d_c` matrix in loop order `n` and homological degree `d`, run:

```
cargo run --release -- n d
```

To see all of the options:
```
cargo run --release -- --help
```

## Pruning a matrix in the SMS format
This finds all of the rows with single nonzero entries and prunes them from the matrix.
```
cargo run --release --bin smsprune -- <matrix file>
```

To see the options:
```
cargo run --release --bin smsprune -- --help
```
In particular, the `-2` flag also prunes the rows with two nonzero entries by adding the columns
together in a way that leaves only one nonzero in that row.

## Finding integral kernel from the `mod p` kernel
The `representatives` binary tries to construct a basis of the integral kernel of a sparse matrix
from the elements of the kernel modulo p.
```
cargo run --release --bin representatives -- <matrix file> <kernel vectors modulo p> <p>
```

## License
AGPL-3.0

