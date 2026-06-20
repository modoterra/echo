# Concurrency Runtime Model

## Goal

Echo's concurrency model should be owned by Echo, not by Rust async runtimes.

The language-level vocabulary is:

- `defer`: create a non-started `EchoTask`.
- `run`: start lightweight Echo task/fiber work.
- `fork`: start OS-thread-backed parallel work.
- `spawn`: start a child process.
- `join`: wait for result or completion.
- `loop`: repeat a block until the body terminates it.
- `gen fn`: declare a generator function that may `yield`.

Do not introduce `async` / `await` in this phase. Do not use `spawn` for lightweight tasks. Do not introduce `EchoJob`.

Core rule:

```text
Mio wakes sockets.
Echo wakes tasks.
```

Use this boundary when choosing where behavior belongs: readiness and OS polling stay in `echo_runtime`; task state and scheduling stay in Echo-owned runtime concepts.

Mio, Crossbeam, Parking Lot, Slab, and similar crates are implementation details inside `echo_runtime`. Echo AST, parser, codegen, and user-facing APIs must not expose those crates or Rust `Future` concepts.

## Runtime Concepts

The runtime concepts are:

- `EchoTask`: lightweight Echo task, eventually backed by PHP-compatible stackful fibers.
- `EchoTaskGroup`: ordered collection of tasks/results for a future task-group expression.
- `EchoThread`: OS-thread-backed parallel work handle.
- `EchoProcess`: child-process handle.

There is no `EchoJob` type. Deferred work is represented by `EchoTask` with state `Deferred`.

## Keyword Semantics

### `loop`

`loop { ... }` is Echo's unconditional loop construct. It does not take a condition; termination belongs in the body.

```php
loop {
    let $conn = net.accept($server)

    if $conn is null {
        break
    }

    handle($conn)
}
```

This form is for server and stream loops where the body owns the stop condition. The header stays unconditional, and the body decides whether to break, return, or keep serving work.

Meaning:

- Repeats until the body exits with `break`, `return`, a thrown error, cancellation, or another terminating construct.
- Does not support `loop while ...` or `loop (...)` forms.
- Leaves PHP's `while`, `for`, `foreach`, and `do while` keywords as PHP-compatible syntax.
- May become expression-valued through `break $value`.

```php
fn waitForConnection($server): TcpConnection {
    return loop {
        let $conn = net.accept($server)

        if $conn is not null {
            break $conn
        }
    }
}
```

This example shows why `loop` may become expression-valued: the loop can keep polling until it has the value the caller needs, then return that value through `break`.

### `fn` and `gen fn`

`fn` is Echo's short function form. `gen fn` is the only generator declaration form; Echo does not provide a long `generator function` alias.

```php
fn activeUsers($users): list<User> {
    return $users.filter(fn ($user): bool => $user.active)
}

gen fn connections($server): TcpConnection {
    loop {
        let $conn = net.accept($server)

        if $conn is null {
            break
        }

        yield $conn
    }
}
```

Use `fn` for ordinary functions and `gen fn` only when callers should consume yielded values. The yielded item type remains visible at the source level instead of being hidden behind runtime machinery.

Meaning:

- `fn name(...) { ... }` declares an Echo function.
- `fn (...) => expr` remains closure shorthand.
- `fn (...) { ... }` is a block closure.
- `gen fn name(...): T { ... }` declares a generator function yielding `T` values.
- `yield` is valid only inside `gen fn`; it must not implicitly change a plain function's type.
- The return annotation on `gen fn` names the yielded item type, not a hidden wrapper type in source.

### `defer`

`defer { ... }` creates an `EchoTask<T>` in `Deferred` state.

```php
$task = defer {
    return fetch_user($id)
}
```

This is the scheduling primitive for work that should be described now and started later. It is useful when a caller needs to collect work before deciding how much to run.

Meaning:

- Captures a block/closure.
- Does not schedule or execute it.
- Can later be started with `run $task`.
- Useful for delayed scheduling, batching, pooling, and retrying.

### `run`

`run { ... }` starts lightweight Echo task/fiber work immediately.

```php
$task = run {
    return fetch_user($id)
}

$user = join $task
```

This is the normal lightweight concurrency pattern: start I/O-oriented work, keep the handle, and join where the result is actually needed.

Meaning:

- Returns an `EchoTask<T>` handle.
- Intended for I/O-heavy concurrent work.
- Concurrent but not necessarily parallel.
- May suspend on Echo-aware I/O, timers, `yield`, `join`, or Fiber suspension.

`run $task` starts an existing deferred task.

```php
$task = defer {
    return fetch_user($id)
}

run $task
$user = join $task
```

This separates task construction from task start. Use it when retry, batching, or admission control needs to happen before work becomes runnable.

If the task is already running, finished, or failed, the runtime should report a clear error.

### Task Groups

A future task-group expression should start multiple lightweight tasks concurrently and wait for all. The exact surface syntax is intentionally open because `{}` is the list literal syntax and `run { ... }` already means a single task block.

```php
[$user, $posts] = run all {
    fetch_user($id)
    fetch_posts($id)
}
```

This shape is for independent requests whose results are consumed together. Source order controls result order, even though execution order remains unspecified.

Meaning:

- Starts all entries concurrently.
- Waits for all entries to finish.
- Returns results in source order.
- Execution order is unspecified.
- Result order is stable.
- No explicit `join` is needed for task-group syntax.
- V1 may let all tasks finish and then rethrow the first error if one task fails.
- V1 does not need cancellation.

Internal lowering can model this as a task group:

```text
group = echo_task_group_new()
task_a = echo_task_defer(closure_a)
task_b = echo_task_defer(closure_b)
echo_task_group_add(group, task_a)
echo_task_group_add(group, task_b)
results = echo_task_group_run_and_join(group)
```

The lowering keeps task groups as runtime-owned handles. The compiler can preserve ordering without exposing queues, pollers, or worker internals to source programs.

### `fork`

`fork { ... }` starts OS-thread-backed parallel work.

```php
$worker = fork {
    return Image::resize($path, 1024, 1024)
}

$image = join $worker
```

Use `fork` when the work should actually run in parallel or may block an OS thread. Lightweight Echo tasks should stay on `run`.

Meaning:

- Starts immediately.
- May run in parallel on another CPU core.
- Intended for CPU-heavy or blocking work.
- Runs in the same process with a separate runtime thread context.
- Is not a child process.

### `spawn`

`spawn` is reserved for child processes.

```php
$proc = spawn "worker --queue=emails"
$status = join $proc
```

This is the process boundary. It should be used for external executables, not for Echo task scheduling or thread-backed parallel work.

Meaning:

- Starts a child process.
- Uses a separate process/address space.
- `join` waits for exit status/result.
- Must not be used for lightweight tasks or OS threads.

### `join`

`join` waits for a previously-started handle.

```php
$value = join $task
$result = join $thread
$status = join $proc
```

`join` is intentionally shared across handle kinds, but the runtime behavior depends on the handle. Joining an Echo task should suspend cooperatively when already inside the scheduler.

Meaning:

- `join EchoTask<T> -> T`
- `join EchoThread<T> -> T`
- `join EchoProcess -> process status/result`
- If already finished, return the stored result.
- If failed/threw, rethrow at the join site.
- If called inside an Echo task, suspend the current task rather than blocking the event-loop worker.
- If called at top level, v1 may drive the runtime until completion.

## Current Timer Slice

The current implementation supports a first cooperative timer path for generated `run { ... }` / `defer { ... }` callbacks whose first statement is `time.sleep(<millis>)`:

1. The generated task callback calls `echo_task_sleep_current(millis, continuation)`.
2. The task enters `Waiting(WaitReason::TimerMillis(millis))` and returns a pending sentinel instead of finishing.
3. The lazy event-loop worker schedules the continuation after the timer expires.
4. `join` blocks on task completion; it does not run queued task work itself.

This is a bridge toward stackful fiber suspension. It intentionally does not yet capture arbitrary locals across a sleep point or suspend from the middle of arbitrary userland call stacks.

The syntax-only generator fixture is currently supported when generator functions are declared but not invoked. Full generator iteration semantics are still future work.

Concurrent HTTP over `run { ... }` remains blocked on task-aware networking. Blocking `net.accept()` or `net.read()` inside a task can occupy the current worker instead of suspending the task and waking it through the event loop.

## AST Direction

Add explicit expression forms for these constructs:

```rust
pub enum Expr {
    Defer(DeferExpr),
    Run(RunExpr),
    Fork(ForkExpr),
    Spawn(SpawnExpr),
    Join(JoinExpr),
}

pub struct DeferExpr {
    pub body: Block,
    pub span: Span,
}

pub enum RunExpr {
    Block { body: Block, span: Span },
    Task { expr: Box<Expr>, span: Span },
    Group { entries: Vec<Block>, span: Span },
}

pub struct ForkExpr {
    pub body: Block,
    pub span: Span,
}

pub enum SpawnExpr {
    Command { command: Box<Expr>, span: Span },
}

pub struct JoinExpr {
    pub handle: Box<Expr>,
    pub span: Span,
}
```

These AST nodes keep concurrency constructs explicit after parsing. Later compiler stages should not recover `run`, `fork`, or `join` behavior from generic calls or ad hoc strings.

Parser support should be careful with PHP compatibility, but Echo features are always available. In Echo mode, valid PHP stays valid while `run`, `fork`, `spawn`, and `join` can also be used as Echo syntax where unambiguous. Strict mode may reject unsafe PHP patterns, but it must not be the only way to use Echo concurrency features.

The first parser slices support `run $task`, `fork $worker`, `spawn "cmd"`, `join $task`, and assignment block forms such as `$task = defer { ... };`, `$task = run { ... };`, and `$worker = fork { ... };`. General block expressions in every expression position still need a broader parser refactor.

Examples that AST output should eventually distinguish:

```php
$task = defer { return fetch_user($id); };
run $task;
$user = join $task;
```

This output should identify a deferred task that is started later by `run $task`.

```php
$task = run { return fetch_user($id); };
$user = join $task;
```

This output should identify a task that starts immediately from a block expression.

```php
[$user, $posts] = run [
    { return fetch_user($id); },
    { return fetch_posts($id); },
];
```

This output should preserve grouped entries in source order so later lowering can return ordered results.

```php
$worker = fork { return cpu_work(); };
$result = join $worker;
```

This output should distinguish OS-thread-backed work from lightweight Echo tasks.

```php
$proc = spawn "worker --queue=emails";
$status = join $proc;
```

This output should distinguish child-process execution from both task and thread execution.

## Task State Model

Use one `EchoTask` type with one generic waiting state.

```rust
pub enum TaskState {
    Deferred,
    Runnable,
    Running,
    Waiting(WaitReason),
    Finished(EchoValue),
    Failed(EchoError),
}

pub enum WaitReason {
    Io { token: IoToken, interest: IoInterest },
    Timer(Instant),
    Task(TaskId),
    Thread(ThreadId),
    Process(ProcessId),
    Callback(CallbackId),
}
```

The single `Waiting(WaitReason)` shape keeps the scheduler state machine compact. New wait sources should usually add a `WaitReason`, not a new top-level task state.

Do not add top-level states such as `WaitingIo`, `Sleeping`, `WaitingThread`, or `WaitingProcess`.

Lifecycle examples:

- `defer { ... }`: `Deferred`
- `run { ... }`: `Runnable`, then `Running`
- `run $task`: `Deferred -> Runnable`
- Echo-aware I/O would block: `Running -> Waiting(Io(...))`
- `sleep()`: `Running -> Waiting(Timer(...))`
- `join $task`: `Running -> Waiting(Task(...))`
- `join $thread`: `Running -> Waiting(Thread(...))`
- `join $proc`: `Running -> Waiting(Process(...))`
- readiness/completion event: `Waiting(...) -> Runnable`
- normal return: `Running -> Finished(value)`
- throw/error: `Running -> Failed(error)`

Scheduler invariant:

```text
An EchoTask is always Deferred, Runnable, Running, Waiting, Finished, or Failed.
```

Use this invariant as the quick check for scheduler changes. A task should never need an implicit or side-channel state to explain what the worker may do next.

## Runtime ABI Direction

LLVM-generated code should call Echo runtime ABI functions and know nothing about Mio, Crossbeam, Tokio, Rayon, Rust futures, or runtime internals.

Initial ABI shape:

```c
typedef struct EchoTask EchoTask;
typedef struct EchoTaskGroup EchoTaskGroup;
typedef struct EchoThread EchoThread;
typedef struct EchoProcess EchoProcess;
typedef struct EchoCallable EchoCallable;
typedef struct EchoValue EchoValue;

EchoTask *echo_task_defer(EchoCallable *callable);
EchoTask *echo_task_run(EchoCallable *callable);
EchoTask *echo_task_start(EchoTask *task);
EchoValue echo_task_join(EchoTask *task);

EchoTaskGroup *echo_task_group_new(void);
void echo_task_group_add(EchoTaskGroup *group, EchoTask *task);
EchoValue echo_task_group_run_and_join(EchoTaskGroup *group);

EchoThread *echo_thread_fork(EchoCallable *callable);
EchoValue echo_thread_join(EchoThread *thread);

EchoProcess *echo_process_spawn(const uint8_t *cmd, uint64_t cmd_len);
EchoValue echo_process_join(EchoProcess *process);
```

This ABI keeps generated LLVM talking to opaque Echo runtime handles. It leaves Mio, queues, fiber storage, and process internals behind the Rust runtime boundary.

Potential LLVM declaration shape:

```llvm
declare ptr @echo_task_defer(ptr)
declare ptr @echo_task_run(ptr)
declare ptr @echo_task_start(ptr)
declare %EchoValue @echo_task_join(ptr)

declare ptr @echo_task_group_new()
declare void @echo_task_group_add(ptr, ptr)
declare %EchoValue @echo_task_group_run_and_join(ptr)

declare ptr @echo_thread_fork(ptr)
declare %EchoValue @echo_thread_join(ptr)

declare ptr @echo_process_spawn(ptr, i64)
declare %EchoValue @echo_process_join(ptr)
```

These declarations are the compiler-facing version of the ABI. They should stay narrow enough that codegen can emit calls without knowing the runtime implementation strategy.

## Lowering Direction

`defer { ... }`:

```llvm
%closure = call ptr @echo_make_closure(...)
%task = call ptr @echo_task_defer(ptr %closure)
```

Lower `defer` to a closure plus a non-started task handle. No scheduling should happen from this sequence.

`run { ... }`:

```llvm
%closure = call ptr @echo_make_closure(...)
%task = call ptr @echo_task_run(ptr %closure)
```

Lower `run` blocks to immediate scheduling through the runtime. The returned value remains a handle, not the task result.

`run $task`:

```llvm
%started = call ptr @echo_task_start(ptr %task)
```

Lower `run $task` as a state transition on an existing deferred handle. Runtime validation should reject handles that are not startable.

`join $task`:

```llvm
%result = call %EchoValue @echo_task_join(ptr %task)
```

Lower `join` to the handle-specific runtime wait point. If called from an Echo task, the runtime can suspend that task instead of blocking the worker.

Task group expression:

```llvm
%group = call ptr @echo_task_group_new()

%closure_a = call ptr @echo_make_closure(...)
%task_a = call ptr @echo_task_defer(ptr %closure_a)
call void @echo_task_group_add(ptr %group, ptr %task_a)

%closure_b = call ptr @echo_make_closure(...)
%task_b = call ptr @echo_task_defer(ptr %closure_b)
call void @echo_task_group_add(ptr %group, ptr %task_b)

%results = call %EchoValue @echo_task_group_run_and_join(ptr %group)
```

Lower task groups by adding deferred tasks in source order, then let the runtime start and join them together. This keeps ordering semantics in one place.

If closure/callable lowering is not ready, parser/AST support can land first and codegen can emit explicit unsupported diagnostics.

## Runtime Crate Layout

The `echo_runtime` crate should grow toward these modules:

```text
abi
error
value
task
task_group
thread
process
sched
poll
poll::mio
io
net
time
```

This layout documents ownership boundaries inside `echo_runtime`. New scheduler, I/O, and process behavior should land in these runtime modules rather than leaking into parser or codegen structures.

Do not add `job.rs`.

Recommended dependency lock-in for `echo_runtime`:

```toml
mio = { version = "1", features = ["os-poll", "net"] }
crossbeam-channel = "0.5"
crossbeam-deque = "0.8"
parking_lot = "0.12"
slab = "0.4"
bytes = "1"
smallvec = "1"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
rustix = { version = "1", features = ["net", "fs", "process", "time"] }
libc = "0.2"
httparse = "1"
http = "1"
```

These dependencies are runtime implementation tools. They should remain contained in `echo_runtime` and should not become source-language concepts or compiler API types.

Dependency purposes:

- `mio`: low-level nonblocking I/O readiness backend.
- `crossbeam-channel`: cross-thread runtime messages.
- `crossbeam-deque`: future work-stealing queues.
- `parking_lot`: internal runtime locks.
- `slab`: stable runtime IDs/handles.
- `bytes`: efficient byte buffers for net/io/http.
- `smallvec`: small inline wake/result/header lists.
- `thiserror`: internal Rust error definitions.
- `tracing` and `tracing-subscriber`: runtime observability.
- `rustix`: low-level OS wrappers where `std`/Mio are insufficient.
- `libc`: contained fallback platform interop.
- `httparse`: future HTTP/1 parsing.
- `http`: future Rust-side HTTP type vocabulary.

Do not add these to the core runtime yet:

- `tokio`
- `smol`
- `async-executor`
- `async-task`
- `hyper`
- `rayon`

Reasoning: Tokio/smol/async-executor schedule Rust futures, but Echo tasks are PHP-compatible stackful execution units. Hyper is too high-level and Tokio-oriented for core runtime. Rayon may be useful later, but `fork` can start with `std::thread`.

## Minimal Event Loop

Echo owns the event loop. Mio is the readiness poller.

The event loop must be allocated on demand, with one lazy event loop per thread. Plain PHP-compatible programs and Echo programs that do not use concurrency keywords or Echo-aware I/O should not pay for scheduler or Mio setup. The compiler/runtime should request the current thread's event loop only when a program uses constructs such as `defer`, `run`, `join`, `fork`, `spawn`, timers, or nonblocking net/I/O APIs.

This keeps normal CLI execution and PHP compatibility fixtures on the direct path while still allowing the runtime to grow an Echo-owned scheduler for concurrent programs.

Minimal worker state:

```rust
pub struct Worker {
    runnable: VecDeque<TaskId>,
    timers: TimerQueue,
    poller: MioPoller,
    tasks: TaskTable,
    shutting_down: bool,
}
```

This worker state is the minimal scheduler inventory: runnable tasks, timers, readiness polling, task storage, and shutdown state.

Minimal resume result:

```rust
pub enum FiberResult {
    Yielded,
    Waiting(WaitReason),
    Finished(EchoValue),
    Failed(EchoError),
}
```

This result tells the worker exactly how to update a task after resuming it. The worker should not infer completion or waiting state from unrelated side effects.

Event-loop structure:

```text
while not shutting down:
  wake expired timers
  run up to MAX_TASKS_PER_TICK runnable tasks
  handle Yielded / Waiting / Finished / Failed
  choose poll timeout
  poll Mio
  convert readiness into runnable tasks
  stop when no work remains
```

This loop shows the intended fairness model: advance timers, run bounded task work, poll I/O, then wake tasks from readiness.

Run a bounded number of tasks per tick so a large runnable queue does not starve sockets/timers. If tasks remain runnable, do a non-blocking I/O poll. If no tasks are runnable, block until the next timer or I/O event.

V1 does not need:

- priorities
- cancellation
- work stealing
- structured concurrency policy
- task-local state
- fiber-local state
- preemptive scheduling

## Fiber Resume Timing

V1 scheduling is cooperative.

Rule:

```text
run is cooperative.
Long CPU-heavy work belongs in fork.
Echo-aware I/O must suspend instead of blocking the OS thread.
```

Use this rule when deciding whether an operation belongs in `run` or `fork`. Work that cannot yield cooperatively should not occupy the lightweight task scheduler.

A task waiting on I/O, timer, task, thread, or process completion becomes runnable when ready. It resumes after earlier runnable tasks get a chance to run. A task that never yields can block its worker.

Future improvement: compiler/runtime safepoints in loops or function prologues.

```llvm
loop:
  call void @echo_runtime_safepoint()
  ; loop body
  br label %loop
```

This is the future compiler hook for cooperative preemption. It should wait until the runtime state model is stable enough to make safepoints predictable.

Do not implement safepoints until the scheduler/fiber model is stable.

## Asynchronous Callbacks

Callbacks are not a separate execution model.

Rule:

```text
All asynchronous callbacks run as EchoTasks.
```

This keeps callbacks inside the same scheduling model as user-created tasks. There should not be a second callback runner with separate lifetime rules.

Future behavior:

```text
Mio reports fd readable.
Runtime finds callback.
Runtime creates EchoTask for callback.
Scheduler runs that task.
```

This sequence converts readiness into normal Echo work. It prevents I/O callbacks from bypassing task state, diagnostics, and cancellation policy later.

This keeps one executable unit: `EchoTask`.

## Minimal Types

Initial runtime placeholders:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TaskId(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ThreadId(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProcessId(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TimerId(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StreamId(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IoToken(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CallbackId(pub usize);
```

These IDs are internal handles for runtime tables. Source programs and compiler AST nodes should not depend on their concrete representation.

Minimal task table:

```rust
pub struct TaskTable {
    tasks: slab::Slab<Task>,
}

pub struct Task {
    state: TaskState,
    // Later:
    // fiber: EchoFiber,
    // callable: EchoCallable,
    // join_waiters: Vec<TaskId>,
    // result: Option<Result<EchoValue, EchoError>>,
}
```

The task table centralizes task storage and later fiber/callable/result bookkeeping. Scheduler code should pass stable IDs rather than borrowing task internals across waits.

Minimal Mio wrapper:

```rust
pub struct MioPoller {
    poll: mio::Poll,
    events: mio::Events,
}

impl MioPoller {
    pub fn new() -> RuntimeResult<Self>;
    pub fn poll(&mut self, timeout: Option<Duration>) -> RuntimeResult<Vec<ReadyEvent>>;
}
```

This wrapper isolates Mio from the rest of the runtime. Other modules should consume readiness events, not Mio-specific types.

## First Implementation Slices

1. Add lexer/parser recognition for `defer`, `run`, `fork`, `spawn`, and `join`, preserving PHP compatibility through contextual parsing where possible.

2. Add AST nodes for the concurrency expressions and parser tests/fixtures that verify AST output.

3. Add runtime modules and placeholder types for tasks, groups, threads, processes, scheduler, and Mio polling. Do not add `job`.

4. Add locked runtime dependencies, even if some are initially unused.

5. Add a scheduler skeleton with unit tests for queue/state behavior using temporary stub task bodies.

6. Add a minimal `MioPoller` wrapper that compiles and can poll with a timeout.

7. Add codegen stubs or explicit unsupported diagnostics for new expressions until closure/callable lowering is ready.

## Acceptance Criteria

Parser/AST acceptance:

- `defer { ... }`, `run $task`, and `join $task` produce distinct AST nodes.
- `run { ... }` produces `RunExpr::Block`.
- `run [ ... ]` produces `RunExpr::Group` with ordered entries.
- `fork { ... }` produces `ForkExpr`.
- `spawn "command"` produces `SpawnExpr::Command`.

Runtime acceptance:

- `cargo check` passes with the runtime crate and locked dependencies.
- Runtime modules exist for `abi`, `error`, `value`, `task`, `task_group`, `thread`, `process`, `sched`, `poll`, `poll::mio`, `io`, `net`, and `time`.
- There is no `EchoJob` type and no `job` module.
- The scheduler skeleton compiles and models `TaskState::Waiting(WaitReason)`.
- Crate-specific types remain behind runtime/internal Rust boundaries.
