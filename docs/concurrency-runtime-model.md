# Concurrency Runtime Model

## Goal

Echo's concurrency model should be owned by Echo, not by Rust async runtimes.

The language-level vocabulary is:

- `defer`: create a non-started `EchoTask`.
- `run`: start lightweight Echo task/fiber work.
- `fork`: start OS-thread-backed parallel work.
- `spawn`: start a child process.
- `join`: wait for result or completion.

Do not introduce `async` / `await` in this phase. Do not use `spawn` for lightweight tasks. Do not introduce `EchoJob`.

Core rule:

```text
Mio wakes sockets.
Echo wakes tasks.
```

Mio, Crossbeam, Parking Lot, Slab, and similar crates are implementation details inside `echo_runtime`. Echo AST, parser, codegen, and user-facing APIs must not expose those crates or Rust `Future` concepts.

## Runtime Concepts

The runtime concepts are:

- `EchoTask`: lightweight Echo task, eventually backed by PHP-compatible stackful fibers.
- `EchoTaskGroup`: ordered collection of tasks/results for a future task-group expression.
- `EchoThread`: OS-thread-backed parallel work handle.
- `EchoProcess`: child-process handle.

There is no `EchoJob` type. Deferred work is represented by `EchoTask` with state `Deferred`.

## Keyword Semantics

### `defer`

`defer { ... }` creates an `EchoTask<T>` in `Deferred` state.

```php
$task = defer {
    return fetch_user($id)
}
```

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

If the task is already running, finished, or failed, the runtime should report a clear error.

### Task Groups

A future task-group expression should start multiple lightweight tasks concurrently and wait for all. The exact surface syntax is intentionally open because `{}` is the list literal syntax and `run { ... }` already means a single task block.

```php
[$user, $posts] = run all {
    fetch_user($id)
    fetch_posts($id)
}
```

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

### `fork`

`fork { ... }` starts OS-thread-backed parallel work.

```php
$worker = fork {
    return Image::resize($path, 1024, 1024)
}

$image = join $worker
```

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

Meaning:

- `join EchoTask<T> -> T`
- `join EchoThread<T> -> T`
- `join EchoProcess -> process status/result`
- If already finished, return the stored result.
- If failed/threw, rethrow at the join site.
- If called inside an Echo task, suspend the current task rather than blocking the event-loop worker.
- If called at top level, v1 may drive the runtime until completion.

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

Parser support should be careful with PHP compatibility, but Echo features are always available. In Echo mode, valid PHP stays valid while `run`, `fork`, `spawn`, and `join` can also be used as Echo syntax where unambiguous. Strict mode may reject unsafe PHP patterns, but it must not be the only way to use Echo concurrency features.

The first parser slices support `run $task`, `fork $worker`, `spawn "cmd"`, `join $task`, and assignment block forms such as `$task = defer { ... };`, `$task = run { ... };`, and `$worker = fork { ... };`. General block expressions in every expression position still need a broader parser refactor.

Examples that AST output should eventually distinguish:

```php
$task = defer { return fetch_user($id); };
run $task;
$user = join $task;
```

```php
$task = run { return fetch_user($id); };
$user = join $task;
```

```php
[$user, $posts] = run [
    { return fetch_user($id); },
    { return fetch_posts($id); },
];
```

```php
$worker = fork { return cpu_work(); };
$result = join $worker;
```

```php
$proc = spawn "worker --queue=emails";
$status = join $proc;
```

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

## Lowering Direction

`defer { ... }`:

```llvm
%closure = call ptr @echo_make_closure(...)
%task = call ptr @echo_task_defer(ptr %closure)
```

`run { ... }`:

```llvm
%closure = call ptr @echo_make_closure(...)
%task = call ptr @echo_task_run(ptr %closure)
```

`run $task`:

```llvm
%started = call ptr @echo_task_start(ptr %task)
```

`join $task`:

```llvm
%result = call %EchoValue @echo_task_join(ptr %task)
```

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

Minimal resume result:

```rust
pub enum FiberResult {
    Yielded,
    Waiting(WaitReason),
    Finished(EchoValue),
    Failed(EchoError),
}
```

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

A task waiting on I/O, timer, task, thread, or process completion becomes runnable when ready. It resumes after earlier runnable tasks get a chance to run. A task that never yields can block its worker.

Future improvement: compiler/runtime safepoints in loops or function prologues.

```llvm
loop:
  call void @echo_runtime_safepoint()
  ; loop body
  br label %loop
```

Do not implement safepoints until the scheduler/fiber model is stable.

## Asynchronous Callbacks

Callbacks are not a separate execution model.

Rule:

```text
All asynchronous callbacks run as EchoTasks.
```

Future behavior:

```text
Mio reports fd readable.
Runtime finds callback.
Runtime creates EchoTask for callback.
Scheduler runs that task.
```

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
