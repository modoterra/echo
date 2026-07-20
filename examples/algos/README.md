# `examples/algos/`

Famous, simple algorithms written in Echo. These programs exercise the locked
language surface (`docs/syntax.md`).

```bash
cargo run -p xo -- check examples/algos/factorial.echo
# when the program is within codegen v1:
# cargo run -p xo -- run examples/algos/factorial.echo
```

| File | Algorithms |
|------|------------|
| [`factorial.echo`](factorial.echo) | Iterative and recursive factorial (result for negatives) |
| [`fibonacci.echo`](fibonacci.echo) | Iterative Fibonacci + nth term |
| [`gcd.echo`](gcd.echo) | Euclid GCD, LCM, Stein binary GCD |
| [`collatz.echo`](collatz.echo) | Collatz / 3n+1 step count |
| [`power.echo`](power.echo) | Exponentiation by squaring |
| [`primes.echo`](primes.echo) | Trial division + sieve of Eratosthenes |
| [`search.echo`](search.echo) | Linear search + binary search |
| [`sort.echo`](sort.echo) | Bubble sort + insertion sort (in place) |
| [`list.echo`](list.echo) | Count, sum, max, reverse, membership |
| [`fizzbuzz.echo`](fizzbuzz.echo) | Classic FizzBuzz (kind codes + print) |
| [`hanoi.echo`](hanoi.echo) | Tower of Hanoi (recursive move count) |
| [`digits.echo`](digits.echo) | Digit sum, reverse digits, numeric palindrome |

## Constraints (current toolchain)

- Prefer **`xo check`** for these until strings / std bridges / more list ops land
  on the run path.
- Lists are **literals** (and bound list values) with index / for-in; no `append`
  yet. Helpers that need a length use a count loop over `* item : xs`.
- `std/io` print/log are stubs until runtime bridges land; call sites still
  exercise the API shape.
