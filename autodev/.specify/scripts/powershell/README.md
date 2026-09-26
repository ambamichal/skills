> Historical documentation: the source preview has known bugs and unverified behavior. Read the repository README and docs/KNOWN-LIMITATIONS.md before use.

# AutoDev Loop

Continuous development automation for AutoDev. Monitors `main` branch, spawns Claude Code windows, and cycles through GitHub issues automatically.

## Quick Start

```powershell
# From repo directory
.\Start-AutoDevLoop.ps1

# From anywhere
powershell -File "C:\path\to\repo\.specify\scripts\powershell\Start-AutoDevLoop.ps1" -RepoPath "C:\path\to\repo"
```

## How It Works

```
+---------------------------------------------------------------------+
|  AutoDevLoop (this script - runs in background)                     |
|                                                                     |
|  +--------------+    +------------------+    +------------------+   |
|  | Detect       |--->| Spawn new        |--->| /autodev runs    |   |
|  | merge to     |    | Claude CLI       |    | creates PR       |   |
|  | main         |    | window           |    | exits            |   |
|  +--------------+    +------------------+    +------------------+   |
|         ^                                           |               |
|         |                                           v               |
|  +------+-------+                         +------------------+      |
|  | User merges  |<------------------------| User gets phone  |      |
|  | PR manually  |                         | notification     |      |
|  +--------------+                         +------------------+      |
+---------------------------------------------------------------------+
```

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `-RepoPath` | string | `.` (current dir) | Path to the repository. Auto-detects git root. |
| `-PollInterval` | int | `30` | Seconds between merge checks. |
| `-MaxCycles` | int | `0` (unlimited) | Maximum development cycles. 0 = infinite. |
| `-DryRun` | switch | `$false` | Preview mode - shows what would happen without executing. |
| `-UseWindowsTerminal` | switch | `$false` | Use Windows Terminal (`wt`) instead of `cmd.exe`. |

## Examples

### Basic usage (from repo directory)
```powershell
.\Start-AutoDevLoop.ps1
```

### Run from any directory
```powershell
powershell -File "C:\Projects\repo\.specify\scripts\powershell\Start-AutoDevLoop.ps1" -RepoPath "C:\Projects\repo"
```

### Limit to 5 cycles
```powershell
.\Start-AutoDevLoop.ps1 -MaxCycles 5
```

### Preview mode (no execution)
```powershell
.\Start-AutoDevLoop.ps1 -DryRun
```

### Slower polling (60 seconds)
```powershell
.\Start-AutoDevLoop.ps1 -PollInterval 60
```

## Cycle Status Messages

| Color | Status | Meaning |
|-------|--------|--------|
| Green | `COMPLETE` | Claude finished successfully, PR created |
| Red | `INTERRUPTED` | Window was closed manually or error occurred |
| Yellow | `TIMED OUT` | Claude exceeded 2-hour timeout |

## Requirements

- **Git** - Version control
- **GitHub CLI** (`gh`) - PR creation and merge detection
- **Claude Code CLI** (`claude`) - AI development automation

## Logs

Logs are saved to:
```
.specify/logs/autodev-loop-YYYYMMDD-HHmmss.log
```

## Tips

- First run asks if you want to start immediately or wait for a merge
- Press `Ctrl+C` to stop the loop gracefully
- Phone notifications require GitHub Mobile app with PR review notifications enabled
- Script auto-detects git root even if run from subdirectory
