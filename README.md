# shamir

Shamir's Secret Sharing over a prime field, implemented from scratch in Rust.

A secret is split into `n` shares, any `t` of which are enough to recover it.
With fewer than `t` shares, every possible secret remains equally likely, so
they reveal nothing at all.

## Example

```rust
use shamir::{reconstruct, split};

const Q: u64 = 65_537;

let mut rng = rand::rng();

// Split the secret into 5 shares, 3 of which are needed
let shares = split(1234, 3, 5, Q, &mut rng)?;

// Any three of them recover it
let secret = reconstruct(&shares[..3], Q)?;
assert_eq!(secret, 1234);
```

## How it works

The secret becomes the constant term of a random polynomial of degree `t - 1`
over the prime field defined by `q`. Each share is a point on that polynomial,
evaluated at `x = 1..=n`. The point at `x = 0` is never handed out, because
that is the secret itself.

Reconstruction interpolates the polynomial at `x = 0` using Lagrange weights.
This requires dividing by differences of the x values, and division in a finite
field means multiplying by a modular inverse — which is why `q` has to be prime.

The modular arithmetic is written from scratch: addition, subtraction and
multiplication in widened integers to avoid overflow, exponentiation via
square-and-multiply, and inversion via Fermat's little theorem. The only
dependency is `rand`, for the randomness of the coefficients.

## Limitations

This is a learning project and not meant for production use:

- Secrets are limited to `u64` and must be smaller than `q`.
- The implementation is not constant-time, so its running time may leak
  information about the values it processes.
- `q` is not checked for primality. A composite modulus silently produces
  wrong results in release builds.
- Choosing, storing and distributing `q` is left entirely to the caller.

For real-world use, prefer an audited crate.

## Development

```sh
cargo test          # unit tests, integration tests and doc tests
cargo clippy        # lints
cargo doc --open    # API documentation
```

## License

Licensed under either of MIT or Apache-2.0, at your option.