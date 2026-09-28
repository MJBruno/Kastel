
# PHASE: 1

```markdown
Scheduler + Task lifecycle
        │
        ├── spawn        ✅
        ├── join         ✅
        ├── yield        ✅
        ├── quantum      ✅
        ├── status       ✅
        ├── cancellation ✅
        └── GC           ✅
                ↓

```

# PHASE: 2

```markdown
Channels
        │
        ├── send        ✅
        ├── recv        ✅
        ├── try_recv    ✅
        ├── close       ✅
        └── wakeups     ✅
                ↓
```

# PHASE 3

```markdown
Scheduler robustness
        │
        ├── deadlock detection   ✅
        ├── nested tasks         ⬜
        ├── cancellation edges   ⬜
        └── stress tests         ⬜
                ↓
```

# PHASE 4

```markdown
select
        │
        └── multi-channel wait   ✅
                ↓
```

# PHASE 5

```markdown
Timers
        │
        └── sleep/timeout        ⬜
                ↓
```

# PHASE 6

```markdown
Synchronization
        │
        ├── Mutex                ⬜
        └── WaitGroup            ⬜
                ↓
```

# PHASE 7
```markdown

Blocking I/O architecture
        │
        └── worker pool          ⬜

```