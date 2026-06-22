# Echo Standard Library Time Foundation

This document specifies the target Echo-native `time` standard library surface.
It is a design and implementation planning note; the current implementation only
contains the first `time.sleep` runtime slice. That slice still accepts an
integer millisecond argument internally; the target Echo API replaces that with
`time.sleep(Duration)` once duration literals and opaque time values exist.

Echo time APIs use dot notation:

```echo
time.now()
time.sleep(500ms)
time.duration(seconds: 5)
```

Do not use PHP namespace-call spelling for Echo standard library APIs:

```echo
time\now()   // invalid Echo stdlib style
time\sleep() // invalid Echo stdlib style
```

The slash form remains available for ordinary PHP namespaces where PHP
compatibility requires it. It is not the spelling of Echo-owned standard
library module calls.

Numeric literals are not objects in Echo, so duration construction must not use
numeric receiver methods:

```echo
5.seconds()        // invalid
250.milliseconds() // invalid
```

Use one of the three duration construction forms instead:

```echo
let $literal = 500ms
let $single = time.milliseconds(500)
let $compound = time.duration(milliseconds: 500)
```

These forms keep units explicit and avoid raw numeric sleeps such as
`time.sleep(500)`, where the unit is ambiguous.

## Public Surface Summary

The target module API is:

```echo
fn now(): Instant
fn monotonic(): MonoInstant
fn timer(): Timer

fn sleep(duration: Duration): void

fn unix(seconds: i64): Instant
fn unix_millis(milliseconds: i64): Instant
fn unix_micros(microseconds: i64): Instant
fn unix_nanos(nanoseconds: i128): Instant

fn nanoseconds(value: i64): Duration
fn microseconds(value: i64): Duration
fn milliseconds(value: i64): Duration
fn seconds(value: i64): Duration
fn minutes(value: i64): Duration
fn hours(value: i64): Duration
fn days(value: i64): Duration
fn weeks(value: i64): Duration

fn duration(
    weeks: i64 = 0,
    days: i64 = 0,
    hours: i64 = 0,
    minutes: i64 = 0,
    seconds: i64 = 0,
    milliseconds: i64 = 0,
    microseconds: i64 = 0,
    nanoseconds: i64 = 0,
): Duration

fn period(
    years: i64 = 0,
    months: i64 = 0,
    weeks: i64 = 0,
    days: i64 = 0,
): Period
```

Module functions create values, access clocks, or interact with the runtime.
Receiver methods provide behavior on existing values.

## Module Functions And Receiver Methods

Module-level functions construct values, access clocks, or interact with the
runtime:

```echo
time.now()
time.monotonic()
time.timer()
time.sleep(...)
time.duration(...)
time.period(...)
time.seconds(...)
```

Receiver methods operate on existing values and are defined with `extend`:

```echo
let $now = time.now()
let $unix = $now.to_unix()

let $timer = time.timer()
let $elapsed = $timer.elapsed()
```

Prefer receiver behavior:

```echo
$created_at.to_unix()
$timer.elapsed()
$duration.total_seconds()
```

Do not design value behavior as a module-only surface:

```echo
time.to_unix($created_at)      // invalid style
time.elapsed($timer)           // invalid style
time.total_seconds($duration)  // invalid style
```

Canonical `extend` shape:

```echo
type Instant = {}

extend Instant {
    pub fn to_unix(self): i64 {
        // seconds since Unix epoch
    }
}
```

Construction and factory functions live on the module. Behavior on values lives
on `extend` receiver methods.

## Core Types

The initial `time` module should define these public types:

```echo
namespace time

pub type Instant
pub type MonoInstant
pub type Duration
pub type Period
pub type Timer
```

These types are public but opaque. User code should construct values through
module functions and literals, not by writing internal fields.

Invalid:

```echo
let $bad = time.Instant {
    nanos_since_unix_epoch: 999999999999999999999999999999999
}
```

Use constructors instead:

```echo
let $now = time.now()
let $timeout = time.seconds(5)
let $delay = 500ms
```

Conceptual internal shapes may be record-like:

```echo
type Instant = {
    nanos_since_unix_epoch: i128
}

type Duration = {
    nanos: i128
}

type MonoInstant = {
    ticks: i128
}

type Timer = {
    started: time.MonoInstant
}
```

Those fields are implementation details.

Later `time` work can add `Date`, `Clock`, `DateTime`, `Zone`, and `Span`.
Those calendar and clock abstractions should build on the same split: module
functions construct or access values, while `extend` receiver methods express
behavior on values.

## Instant

`time.Instant` is an exact wall-clock point on the Unix timeline. Use it for
creation time, expiration time, event time, and serialized timestamps.

```echo
let $created_at = time.now()
let $expires_at = $created_at + 30d

if (time.now() >= $expires_at) {
    echo "expired"
}
```

`Instant` is immutable. Adding or subtracting a `Duration` returns a new
`Instant`. Subtracting two `Instant` values returns a `Duration`.

Required arithmetic:

```text
Instant - Instant = Duration
Instant + Duration = Instant
Instant - Duration = Instant
```

Invalid arithmetic:

```echo
time.now() - time.monotonic() // invalid
```

Required module API:

```echo
fn now(): Instant

fn unix(seconds: i64): Instant
fn unix_millis(milliseconds: i64): Instant
fn unix_micros(microseconds: i64): Instant
fn unix_nanos(nanoseconds: i128): Instant
```

Required receiver methods:

```echo
extend Instant {
    pub fn to_unix(self): i64
    pub fn to_unix_millis(self): i64
    pub fn to_unix_micros(self): i64
    pub fn to_unix_nanos(self): i128
}
```

These methods return Unix timeline values in progressively smaller units.
`to_unix_nanos()` preserves the full internal precision; the coarser methods
truncate toward whole units.

`Instant` must not expose calendar fields such as `year`, `month`, `day`,
`hour`, or `minute`; those depend on timezone.

Invalid:

```echo
let $now = time.now()

echo $now.year
echo $now.hour
```

Future timezone-aware form:

```echo
let $now = time.now()
let $dt = $now.in(time.zone("America/New_York"))

echo $dt.year
echo $dt.hour
```

## MonoInstant

`time.MonoInstant` is a monotonic runtime timestamp for elapsed measurement. It
is not convertible to a date or Unix timestamp.

Required module API:

```echo
fn monotonic(): MonoInstant
```

Required arithmetic:

```text
MonoInstant - MonoInstant = Duration
MonoInstant + Duration = MonoInstant
MonoInstant - Duration = MonoInstant
```

Invalid operations:

```echo
time.monotonic().to_unix()      // invalid
time.monotonic().in(time.utc()) // invalid
time.now() - time.monotonic()   // invalid
```

Use `MonoInstant` for benchmarking, elapsed timing, runtime timers, and
timeouts:

```echo
let $start = time.monotonic()

work()

let $elapsed = time.monotonic() - $start

if ($elapsed > 250ms) {
    echo "slow"
}
```

## Duration

`time.Duration` is exact elapsed machine time. It is not calendar-relative
human time. Store it internally as nanoseconds, preferably `i128`.

Duration literals:

```echo
1ns
500us
250ms
5s
10min
2h
7d
1w
```

Use `min` for minutes. Do not support bare `m`:

```echo
10m // invalid; use 10min
```

Do not support months or years as `Duration` literals:

```echo
1mo // invalid as Duration
1y  // invalid as Duration
```

Months and years belong to `time.Period`.

Required single-unit constructors:

```echo
fn nanoseconds(value: i64): Duration
fn microseconds(value: i64): Duration
fn milliseconds(value: i64): Duration
fn seconds(value: i64): Duration
fn minutes(value: i64): Duration
fn hours(value: i64): Duration
fn days(value: i64): Duration
fn weeks(value: i64): Duration
```

Required compound constructor:

```echo
fn duration(
    weeks: i64 = 0,
    days: i64 = 0,
    hours: i64 = 0,
    minutes: i64 = 0,
    seconds: i64 = 0,
    milliseconds: i64 = 0,
    microseconds: i64 = 0,
    nanoseconds: i64 = 0,
): Duration
```

All named parameters are optional and default to zero:

```echo
let $timeout = time.duration(seconds: 5)

let $window = time.duration(
    minutes: 1,
    seconds: 30,
)

let $zero = time.duration()
```

These are equivalent:

```echo
500ms
time.milliseconds(500)
time.duration(milliseconds: 500)
```

Required arithmetic:

```text
Duration + Duration = Duration
Duration - Duration = Duration
Duration * integer = Duration
Duration / integer = Duration
Duration / Duration = numeric ratio
```

Required comparisons:

```text
Duration == Duration
Duration != Duration
Duration < Duration
Duration <= Duration
Duration > Duration
Duration >= Duration
```

Required receiver methods:

```echo
extend Duration {
    pub fn total_nanos(self): i128
    pub fn total_micros(self): i128
    pub fn total_millis(self): i128
    pub fn total_seconds(self): f64
    pub fn whole_seconds(self): i64
    pub fn whole_minutes(self): i64
    pub fn whole_hours(self): i64
    pub fn whole_days(self): i64
    pub fn abs(self): Duration
}
```

`total_*` methods report the whole duration in the requested unit.
`whole_*` methods return whole elapsed units with smaller units discarded.

Example:

```echo
let $elapsed = time.now() - $started

echo $elapsed.total_millis()

if ($elapsed > 250ms) {
    echo "slow"
}
```

## Sleep

`time.sleep` requires a `time.Duration`.

Valid:

```echo
time.sleep(500ms)
time.sleep(time.milliseconds(500))
time.sleep(time.duration(seconds: 5))
```

Invalid:

```echo
time.sleep(500) // invalid: 500 what?
```

Diagnostic:

```text
time.sleep expects time.Duration, found integer.
Use a duration literal like 500ms or a constructor like time.milliseconds(500).
```

## Timer

`time.Timer` uses `time.MonoInstant` internally and is the preferred API for
measuring elapsed time.

Required module API:

```echo
type Timer

fn timer(): Timer
```

Required receiver methods:

```echo
extend Timer {
    pub fn elapsed(self): time.Duration

    pub fn reset(mut self): time.Duration
}
```

Example:

```echo
let $timer = time.timer()

render()

if ($timer.elapsed() > 16ms) {
    echo "slow frame"
}

let $elapsed = $timer.reset()
```

`Timer.elapsed()` returns the duration since the timer started.
`Timer.reset()` returns the elapsed duration and resets the stored start to the
current monotonic time.

## Period

`time.Period` is calendar-relative human time. It is not exact elapsed time.
Use it for months, years, billing cycles, and local calendar movement.

Required module API:

```echo
type Period

fn period(
    years: i64 = 0,
    months: i64 = 0,
    weeks: i64 = 0,
    days: i64 = 0,
): Period
```

All named parameters are optional and default to zero:

```echo
let $next_billing_cycle = time.period(months: 1)
let $next_year = time.period(years: 1)
let $future = time.period(years: 1, months: 6, days: 3)
```

Important distinction:

```echo
let $exactly_24_hours = 1d
let $calendar_tomorrow = time.period(days: 1)
```

`1d` means exact 24-hour `Duration`. `time.period(days: 1)` means a
calendar-aware local day when applied to a future `DateTime` type.

Do not add duration constructors for months or years:

```echo
time.months(1) // invalid
time.years(1)  // invalid
```

Use:

```echo
time.period(months: 1)
time.period(years: 1)
```

Potential future receiver method:

```echo
extend Period {
    pub fn is_zero(self): bool
}
```

Keep the initial `Period` surface minimal until `Date`, `Clock`, `DateTime`,
and `Zone` exist.

## AST And Typing Plan

Duration literals should parse into a dedicated AST node, not as normal integer
literals:

```rust
pub enum Expr {
    DurationLiteral(DurationLiteral),
}

pub struct DurationLiteral {
    pub value: i128,
    pub unit: DurationUnit,
    pub span: Span,
}

pub enum DurationUnit {
    Nanosecond,
    Microsecond,
    Millisecond,
    Second,
    Minute,
    Hour,
    Day,
    Week,
}
```

The lexer/parser should preserve the source value and suffix so diagnostics and
formatting can explain the user's original spelling:

```text
DurationLiteral:
    Integer "ns"
    Integer "us"
    Integer "ms"
    Integer "s"
    Integer "min"
    Integer "h"
    Integer "d"
    Integer "w"
```

The invalid suffixes `m`, `mo`, `y`, `yr`, `year`, and `month` should produce
targeted diagnostics rather than falling through as ordinary identifiers.

Typing rules:

```text
DurationLiteral => time.Duration
time.duration(...) => time.Duration
time.nanoseconds(i64) => time.Duration
time.microseconds(i64) => time.Duration
time.milliseconds(i64) => time.Duration
time.seconds(i64) => time.Duration
time.minutes(i64) => time.Duration
time.hours(i64) => time.Duration
time.days(i64) => time.Duration
time.weeks(i64) => time.Duration
time.period(...) => time.Period
time.now() => time.Instant
time.monotonic() => time.MonoInstant
time.timer() => time.Timer
```

Reject mixed clock arithmetic and calendar/exact-time confusion:

```text
Instant - MonoInstant
MonoInstant - Instant
Instant + Instant
MonoInstant + MonoInstant
Duration + Period
Instant + Period
```

`Instant + Period` can be considered later only after `DateTime` and `Zone`
exist.

## Diagnostics

Invalid numeric-object style:

```echo
5.seconds()
```

Suggested diagnostic:

```text
Numeric literals do not have methods.
Use 5s, time.seconds(5), or time.duration(seconds: 5).
```

Invalid minute suffix:

```echo
10m
```

Suggested diagnostic:

```text
Invalid duration suffix `m`.
Use `min` for minutes, for example 10min.
```

Invalid month/year duration:

```echo
1mo
1y
```

Suggested diagnostic:

```text
Months and years are calendar periods, not exact durations.
Use time.period(months: 1) or time.period(years: 1).
```

Invalid mixed clock subtraction:

```echo
time.now() - time.monotonic()
```

Suggested diagnostic:

```text
Cannot subtract time.MonoInstant from time.Instant.
Use time.timer() or time.monotonic() for elapsed timing.
```

Invalid direct construction of opaque time types:

```echo
let $bad = time.Instant {
    nanos_since_unix_epoch: 123
}
```

Suggested diagnostic:

```text
time.Instant is opaque and cannot be constructed directly.
Use time.now(), time.unix(...), time.unix_millis(...), or another time constructor.
```

Invalid module function style for receiver behavior:

```echo
time.to_unix($instant)
```

Suggested diagnostic:

```text
Use the receiver method form instead: $instant.to_unix().
```

## Implementation Slices

1. Add `DurationLiteral` and `DurationUnit` AST nodes.
2. Parse integer literals followed by `ns`, `us`, `ms`, `s`, `min`, `h`, `d`, or `w`.
3. Add invalid-suffix diagnostics for `m`, `mo`, `y`, `yr`, `year`, and `month`.
4. Add `extend` receiver-method syntax and dot-call typing for time values.
5. Add semantic facts for opaque `time` core types and typed constructor returns.
6. Implement duration constructors and `time.sleep(Duration)` in std/runtime.
7. Add monotonic clock, timer construction, and `Timer.elapsed()` / `Timer.reset()`.
8. Add `Instant` Unix constructors and conversion receiver methods.
9. Add `Period` constructor and keep it separate from `Duration`.

Runtime-backed slices should use system wall-clock time for `time.now()`,
a monotonic clock source for `time.monotonic()`, and exact duration sleeps for
`time.sleep(Duration)`. `Timer` stores a `MonoInstant`; `elapsed()` subtracts it
from the current monotonic instant, and `reset()` returns that elapsed duration
while replacing the stored start instant.

## Runtime And Std Support Plan

The first implementation should keep the public Echo surface in `std/time.echo`
and route privileged operations through approved runtime intrinsics. The public
types stay opaque even if the Rust runtime stores them as nanosecond counters or
monotonic ticks.

| Surface | Required support |
| --- | --- |
| `time.now()` | system wall-clock `Instant` |
| `time.monotonic()` | monotonic `MonoInstant`, never convertible to Unix time |
| `time.sleep(duration)` | accepts only `time.Duration`, not raw integers |
| `time.timer()` | captures a `MonoInstant` internally |
| `time.duration(...)` | optional named units default to zero |
| `time.period(...)` | optional named calendar units default to zero |
| `time.nanoseconds(...)` through `time.weeks(...)` | plural single-unit `Duration` constructors |
| `Instant`, `Duration`, and `Timer` receiver methods | implemented through `extend` method declarations |

Example target stdlib shape:

```echo
namespace time

pub type Duration
pub type Instant
pub type MonoInstant
pub type Period
pub type Timer

pub fn duration(
    weeks: i64 = 0,
    days: i64 = 0,
    hours: i64 = 0,
    minutes: i64 = 0,
    seconds: i64 = 0,
    milliseconds: i64 = 0,
    microseconds: i64 = 0,
    nanoseconds: i64 = 0,
): Duration

extend Timer {
    pub fn elapsed(self): Duration
    pub fn reset(mut self): Duration
}
```

This shape keeps construction and runtime interaction on the module while value
behavior remains on receiver methods.
