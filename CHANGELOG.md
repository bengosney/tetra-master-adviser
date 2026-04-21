## 1.0.2 (2026-04-21)

### Fix

- **ci**: strip directory paths from release zip

## 1.0.1 (2026-04-21)

### Fix

- **ci**: strip directory paths from release zip

## 1.0.1 (2026-04-21)

## 1.0.0 (2026-04-20)

### Feat

- rename project
- **help**: tidy up the help messages
- try and play defensive and add a history
- inital "working" helper

### Fix

- **card**: fix defence logic
- **terminal**: fix the terminal on panic
- **saveing**: save the state to a temp file
- **loading**: clamp to valid bounds
- remove history logging

### Refactor

- replace magic numbers with consts
- remove unused field

### Perf

- **solver**: eliminate per-node hand allocations
