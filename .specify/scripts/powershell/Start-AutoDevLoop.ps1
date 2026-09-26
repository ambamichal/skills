<#
.SYNOPSIS
    Continuous Auto-Development Loop for AutoDev Framework

.DESCRIPTION
    This script creates a continuous development loop that:
    1. Monitors the 'main' branch for new merges (from manually merged PRs)
    2. When a merge is detected, spawns a NEW Claude Code CLI window
    3. Runs /autodev command which ends with a new PR
    4. Waits for user to manually merge the PR (with phone notification)
    5. Detects the merge and repeats the cycle

    WORKFLOW:
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
    |  | User merges  |<------------------------|  User gets       |      |
    |  | PR manually  |     (phone notification)| notification     |      |
    |  +--------------+                         +------------------+      |
    +---------------------------------------------------------------------+

.PARAMETER RepoPath
    Path to the repository. Defaults to current directory.

.PARAMETER PollInterval
    How often to check for merges in seconds. Default: 30 seconds.

.PARAMETER MaxCycles
    Maximum number of development cycles to run. Default: unlimited (0).

.PARAMETER DryRun
    Show what would be done without executing.

.PARAMETER UseWindowsTerminal
    Use Windows Terminal (wt) instead of cmd.exe for new windows.

.EXAMPLE
    .\Start-AutoDevLoop.ps1
    Start the continuous development loop

.EXAMPLE
    .\Start-AutoDevLoop.ps1 -MaxCycles 5
    Run only 5 development cycles

.EXAMPLE
    .\Start-AutoDevLoop.ps1 -DryRun
    Preview what would happen without executing

.NOTES
    Version: 2.0.0
    Author: AutoDev Framework
    Requires: Git, GitHub CLI (gh), Claude Code CLI (claude)
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory = $false)]
    [string]$RepoPath = ".",

    [Parameter(Mandatory = $false)]
    [int]$PollInterval = 30,

    [Parameter(Mandatory = $false)]
    [int]$MaxCycles = 0,

    [Parameter(Mandatory = $false)]
    [switch]$DryRun,

    [Parameter(Mandatory = $false)]
    [switch]$UseWindowsTerminal
)

# Set error action preference
$ErrorActionPreference = "Continue"

# Resolve absolute path - find git root if in subdirectory
$RepoPath = (Resolve-Path $RepoPath).Path
Push-Location $RepoPath
$gitRoot = git rev-parse --show-toplevel 2>$null
if ($LASTEXITCODE -eq 0 -and $gitRoot) {
    $RepoPath = $gitRoot.Replace('/', '\')
}
Pop-Location

# Script state
$script:CycleCount = 0
$script:StartTime = Get-Date
$script:LastKnownMainSha = $null
$script:LogFile = Join-Path $RepoPath ".specify/logs/autodev-loop-$(Get-Date -Format 'yyyyMMdd-HHmmss').log"

#region Logging Functions

function Write-Log {
    param(
        [string]$Message,
        [ValidateSet('INFO', 'SUCCESS', 'WARNING', 'ERROR', 'CYCLE')]
        [string]$Level = 'INFO'
    )

    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $logMessage = "[$timestamp] [$Level] $Message"

    # Ensure log directory exists
    $logDir = Split-Path $script:LogFile -Parent
    if (-not (Test-Path $logDir)) {
        New-Item -ItemType Directory -Path $logDir -Force | Out-Null
    }

    # Write to log file
    Add-Content -Path $script:LogFile -Value $logMessage -ErrorAction SilentlyContinue

    # Write to console with colors
    $color = switch ($Level) {
        'SUCCESS' { 'Green' }
        'WARNING' { 'Yellow' }
        'ERROR' { 'Red' }
        'CYCLE' { 'Cyan' }
        default { 'White' }
    }

    $prefix = switch ($Level) {
        'SUCCESS' { '[OK]' }
        'WARNING' { '[!!]' }
        'ERROR' { '[XX]' }
        'CYCLE' { '[=>]' }
        default { '[--]' }
    }

    Write-Host "$prefix $Message" -ForegroundColor $color
}

function Show-Banner {
    Write-Host ""
    Write-Host "    ==========================================================================" -ForegroundColor DarkCyan
    Write-Host ""
    Write-Host "                     ___         __         ____                              " -ForegroundColor Cyan
    Write-Host "                    /   | __  __/ /_____   / __ \ ___  _   __                 " -ForegroundColor Cyan
    Write-Host "                   / /| |/ / / / __/ __ \ / / / // _ \| | / /                 " -ForegroundColor Cyan
    Write-Host "                  / ___ / /_/ / /_/ /_/ // /_/ //  __/| |/ /                  " -ForegroundColor Cyan
    Write-Host "                 /_/  |_\__,_/\__/\____//_____/ \___/ |___/                   " -ForegroundColor Cyan
    #Write-Host ""
    #Write-Host "                       Continuous Development Loop v0.1                       " -ForegroundColor White
    Write-Host "                              AutoDev Framework                               " -ForegroundColor DarkGray
    Write-Host ""
    Write-Host "    ==========================================================================" -ForegroundColor DarkCyan
    Write-Host ""
    Write-Host "    Repository:     $RepoPath" -ForegroundColor Gray
    Write-Host "    Log file:       $script:LogFile" -ForegroundColor Gray
    Write-Host "    Poll interval:  $PollInterval seconds" -ForegroundColor Gray
    if ($MaxCycles -gt 0) {
        Write-Host "    Max cycles:     $MaxCycles" -ForegroundColor Gray
    } else {
        Write-Host "    Max cycles:     Unlimited" -ForegroundColor Gray
    }
    Write-Host ""
    Write-Host "    How it works:" -ForegroundColor Yellow
    Write-Host "      1. Monitors 'main' branch for new merges" -ForegroundColor Gray
    Write-Host "      2. Spawns NEW Claude Code window with /autodev" -ForegroundColor Gray
    Write-Host "      3. /autodev creates PR -> you get phone notification" -ForegroundColor Gray
    Write-Host "      4. You manually review & merge -> cycle repeats" -ForegroundColor Gray
    Write-Host ""
}

function Show-Status {
    param(
        [string]$Status,
        [string]$Details = ""
    )

    $elapsed = (Get-Date) - $script:StartTime
    $elapsedStr = "{0:hh\:mm\:ss}" -f $elapsed

    Write-Host ""
    Write-Host "------------------------------------------------------------------------" -ForegroundColor DarkGray
    Write-Host " Status: $Status" -ForegroundColor Cyan
    if ($Details) {
        Write-Host " $Details" -ForegroundColor Gray
    }
    Write-Host " Cycle: $($script:CycleCount) | Elapsed: $elapsedStr" -ForegroundColor Gray
    Write-Host "------------------------------------------------------------------------" -ForegroundColor DarkGray
    Write-Host ""
}

#endregion

#region Git Functions

function Get-MainBranchSha {
    <#
    .SYNOPSIS
        Get the current SHA of the main branch (from remote)
    #>

    try {
        Push-Location $RepoPath

        # Fetch latest from remote (silent)
        git fetch origin main 2>$null | Out-Null

        # Get the SHA of origin/main
        $sha = git rev-parse origin/main 2>$null
        if ($LASTEXITCODE -ne 0) {
            Write-Log "Failed to get main branch SHA" -Level ERROR
            return $null
        }

        return $sha.Trim()
    }
    catch {
        Write-Log "Error getting main SHA: $_" -Level ERROR
        return $null
    }
    finally {
        Pop-Location
    }
}

function Get-LatestMergeInfo {
    <#
    .SYNOPSIS
        Get information about the latest merge to main
    #>

    try {
        Push-Location $RepoPath

        # Get latest commit info on main
        $commitInfo = git log origin/main -1 --format="%H|%s|%an|%ar" 2>$null
        if ($LASTEXITCODE -ne 0 -or -not $commitInfo) {
            return $null
        }

        $parts = $commitInfo -split '\|'
        return @{
            Sha = $parts[0]
            Message = $parts[1]
            Author = $parts[2]
            TimeAgo = $parts[3]
        }
    }
    catch {
        return $null
    }
    finally {
        Pop-Location
    }
}

function Sync-MainBranch {
    <#
    .SYNOPSIS
        Sync local main branch with remote
    #>

    try {
        Push-Location $RepoPath

        Write-Log "Syncing main branch..."

        # Checkout main
        git checkout main 2>&1 | Out-Null
        if ($LASTEXITCODE -ne 0) {
            Write-Log "Failed to checkout main" -Level ERROR
            return $false
        }

        # Pull latest
        $pullResult = git pull origin main 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Log "Failed to pull main: $pullResult" -Level ERROR
            return $false
        }

        # Prune old branches
        git fetch --prune 2>&1 | Out-Null

        Write-Log "Main branch synced successfully" -Level SUCCESS
        return $true
    }
    catch {
        Write-Log "Error syncing main: $_" -Level ERROR
        return $false
    }
    finally {
        Pop-Location
    }
}

function Test-NewMergeDetected {
    <#
    .SYNOPSIS
        Check if there's a new merge on main since last check
    #>

    $currentSha = Get-MainBranchSha

    if ($null -eq $currentSha) {
        return $false
    }

    # First run - initialize baseline
    if ($null -eq $script:LastKnownMainSha) {
        $script:LastKnownMainSha = $currentSha
        $mergeInfo = Get-LatestMergeInfo
        Write-Log "Initialized baseline SHA: $($currentSha.Substring(0, 8))" -Level INFO
        if ($mergeInfo) {
            Write-Log "Latest commit: $($mergeInfo.Message)" -Level INFO
        }
        return $false
    }

    # Check if SHA changed (new merge detected)
    if ($currentSha -ne $script:LastKnownMainSha) {
        $mergeInfo = Get-LatestMergeInfo
        Write-Log "*** NEW MERGE DETECTED! ***" -Level SUCCESS
        if ($mergeInfo) {
            Write-Log "  Commit: $($mergeInfo.Message)" -Level INFO
            Write-Log "  Author: $($mergeInfo.Author)" -Level INFO
            Write-Log "  Time: $($mergeInfo.TimeAgo)" -Level INFO
        }
        $script:LastKnownMainSha = $currentSha
        return $true
    }

    return $false
}

#endregion

#region Claude Code Functions

function Test-ClaudeCodeInstalled {
    <#
    .SYNOPSIS
        Check if Claude Code CLI is installed
    #>

    try {
        $version = & claude --version 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Log "Claude Code CLI: $version"
            return $true
        }
    }
    catch {
        # Command not found
    }

    Write-Log "Claude Code CLI not found. Please install it first." -Level ERROR
    Write-Log "Install with: npm install -g @anthropic-ai/claude-code" -Level INFO
    return $false
}

function Start-ClaudeAutodevWindow {
    <#
    .SYNOPSIS
        Spawn a new visible CMD window and run Claude Code with /autodev
        Full stdout/stderr is captured to a log file for later review.
    #>

    Write-Log "Spawning new Claude Code window..." -Level CYCLE

    if ($DryRun) {
        Write-Log "[DRY RUN] Would spawn: cmd /k claude --dangerously-skip-permissions -p /autodev" -Level WARNING
        return $null
    }

    try {
        $cycleNum = $script:CycleCount + 1

        # Generate unique log filename for this autodev run
        $timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
        $claudeLogFile = Join-Path $RepoPath ".specify/logs/autodev-run-$timestamp-cycle$cycleNum.log"

        # Ensure log directory exists
        $logDir = Split-Path $claudeLogFile -Parent
        if (-not (Test-Path $logDir)) {
            New-Item -ItemType Directory -Path $logDir -Force | Out-Null
        }

        Write-Log "Claude output will be saved to: $claudeLogFile" -Level INFO

        # Build PowerShell script block to run in new window
        # This approach uses PowerShell's Tee-Object to both display AND log output
        $psScriptBlock = @"
`$Host.UI.RawUI.WindowTitle = 'AutoDev Cycle #$cycleNum'
Set-Location -Path '$RepoPath'

Write-Host ''
Write-Host '========================================'
Write-Host '  AUTODEV CYCLE #$cycleNum'
Write-Host '  Working dir: $RepoPath'
Write-Host '  Log file: $claudeLogFile'
Write-Host '========================================'
Write-Host ''

# Start logging header
@'
================================================================================
AUTODEV CYCLE #$cycleNum
Started: `$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')
Working dir: $RepoPath
================================================================================

'@ | Out-File -FilePath '$claudeLogFile' -Encoding UTF8

# Run Claude and capture ALL output (stdout + stderr) to both console and log file
# Using script block with 2>&1 to merge stderr into stdout, then Tee-Object
try {
    & claude --dangerously-skip-permissions -p '/autodev' 2>&1 | ForEach-Object {
        `$line = `$_.ToString()
        Write-Host `$line
        `$line | Out-File -FilePath '$claudeLogFile' -Append -Encoding UTF8
    }
    `$exitCode = `$LASTEXITCODE
}
catch {
    `$errorMsg = "ERROR: `$(`$_.Exception.Message)"
    Write-Host `$errorMsg -ForegroundColor Red
    `$errorMsg | Out-File -FilePath '$claudeLogFile' -Append -Encoding UTF8
    `$exitCode = 1
}

# Log footer
@'

================================================================================
AUTODEV CYCLE #$cycleNum COMPLETED
Finished: `$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')
Exit code: `$exitCode
================================================================================
'@ | Out-File -FilePath '$claudeLogFile' -Append -Encoding UTF8

Write-Host ''
Write-Host '========================================'
Write-Host '  CYCLE COMPLETE - Window will close'
Write-Host '  Log saved to: $claudeLogFile'
Write-Host '========================================'
Start-Sleep -Seconds 5
exit `$exitCode
"@

        # Encode the script block to avoid escaping issues
        $bytes = [System.Text.Encoding]::Unicode.GetBytes($psScriptBlock)
        $encodedCommand = [Convert]::ToBase64String($bytes)

        # Launch new PowerShell window with encoded command
        $process = Start-Process -FilePath "powershell.exe" `
            -ArgumentList "-NoProfile", "-ExecutionPolicy", "Bypass", "-EncodedCommand", $encodedCommand `
            -PassThru

        Write-Log "New PowerShell window spawned (PID: $($process.Id))" -Level SUCCESS
        Write-Log "Log file: $claudeLogFile" -Level INFO
        return $process
    }
    catch {
        Write-Log "Error spawning Claude window: $_" -Level ERROR
        return $null
    }
}

function Wait-ForClaudeCompletion {
    <#
    .SYNOPSIS
        Wait for the Claude Code window to complete
    .PARAMETER Process
        The process object of the Claude window
    .PARAMETER TimeoutMinutes
        Maximum time to wait in minutes (default: 120 = 2 hours)
    #>
    param(
        [System.Diagnostics.Process]$Process,
        [int]$TimeoutMinutes = 120
    )

    if ($null -eq $Process -or $DryRun) {
        Write-Log "[DRY RUN] Would wait for Claude to complete" -Level WARNING
        Start-Sleep -Seconds 5
        return $true
    }

    Write-Log "Waiting for Claude Code to complete (PID: $($Process.Id))..."
    Write-Log "  Timeout: $TimeoutMinutes minutes" -Level INFO
    Write-Log "  The Claude window is running /autodev" -Level INFO
    Write-Log "  You will get a phone notification when PR is created" -Level INFO

    $startWait = Get-Date
    $timeoutMs = $TimeoutMinutes * 60 * 1000

    try {
        $exited = $Process.WaitForExit($timeoutMs)

        if (-not $exited) {
            Write-Log "Claude process timed out after $TimeoutMinutes minutes" -Level WARNING
            return $false
        }

        $elapsed = (Get-Date) - $startWait
        Write-Log "Claude completed in $("{0:mm\:ss}" -f $elapsed)" -Level SUCCESS

        # Check exit code
        if ($Process.ExitCode -eq 0) {
            Write-Log "Claude exited successfully (code 0)" -Level SUCCESS
            return $true
        }
        else {
            Write-Log "Claude exited with code: $($Process.ExitCode)" -Level WARNING
            return $true  # Still continue - might have created PR
        }
    }
    catch {
        Write-Log "Error waiting for Claude: $_" -Level ERROR
        return $false
    }
}

#endregion

#region Main Loop

function Start-DevLoop {
    <#
    .SYNOPSIS
        Main development loop - monitors merges and spawns Claude windows
    #>

    Show-Banner

    # Verify prerequisites
    Write-Log "Checking prerequisites..."

    # Check GitHub CLI
    $ghVersion = gh --version 2>$null | Select-Object -First 1
    if ($LASTEXITCODE -ne 0) {
        Write-Log "GitHub CLI (gh) not found. Please install it first." -Level ERROR
        return
    }
    Write-Log "GitHub CLI: $ghVersion"

    # Check Claude Code CLI
    if (-not (Test-ClaudeCodeInstalled)) {
        return
    }

    # Verify repo
    Push-Location $RepoPath
    $isGitRepo = git rev-parse --git-dir 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Log "Not a git repository: $RepoPath" -Level ERROR
        Pop-Location
        return
    }
    Write-Log "Repository verified: $RepoPath"
    Pop-Location

    # Sync main branch first
    if (-not (Sync-MainBranch)) {
        Write-Log "Failed to sync main branch. Please check manually." -Level ERROR
        return
    }

    # Initialize baseline SHA
    $initialSha = Get-MainBranchSha
    if ($null -eq $initialSha) {
        Write-Log "Failed to get initial main branch SHA" -Level ERROR
        return
    }
    $script:LastKnownMainSha = $initialSha

    $mergeInfo = Get-LatestMergeInfo
    Write-Host ""
    Write-Host "+================================================================+" -ForegroundColor Green
    Write-Host "|  AUTODEV LOOP STARTED                                          |" -ForegroundColor Green
    Write-Host "+================================================================+" -ForegroundColor Green
    Write-Host "|  Current main SHA: $($initialSha.Substring(0, 8))...                              |" -ForegroundColor Green
    if ($mergeInfo) {
        $truncatedMsg = if ($mergeInfo.Message.Length -gt 45) { $mergeInfo.Message.Substring(0, 42) + "..." } else { $mergeInfo.Message.PadRight(45) }
        Write-Host "|  Latest commit: $truncatedMsg |" -ForegroundColor Green
    }
    Write-Host "|                                                                |" -ForegroundColor Green
    Write-Host "|  Monitoring for new merges to main...                          |" -ForegroundColor Green
    Write-Host "|  Press Ctrl+C to stop                                          |" -ForegroundColor Green
    Write-Host "+================================================================+" -ForegroundColor Green
    Write-Host ""

    # Ask if user wants to start first cycle immediately
    Write-Host "Do you want to start the first /autodev cycle immediately? (Y/N)" -ForegroundColor Yellow
    $startNow = Read-Host

    if ($startNow -eq 'Y' -or $startNow -eq 'y') {
        Write-Log "Starting first cycle immediately..." -Level CYCLE
        $runCycle = $true
    }
    else {
        Write-Log "Waiting for first merge to main to trigger cycle..." -Level INFO
        $runCycle = $false
    }

    # Main loop
    while ($true) {
        # Check max cycles
        if ($MaxCycles -gt 0 -and $script:CycleCount -ge $MaxCycles) {
            Write-Log "Reached maximum cycles ($MaxCycles). Stopping." -Level SUCCESS
            break
        }

        # Run cycle if triggered
        if ($runCycle) {
            $script:CycleCount++

            Write-Host ""
            Write-Host "+================================================================+" -ForegroundColor Cyan
            Write-Host "|  STARTING DEVELOPMENT CYCLE #$($script:CycleCount.ToString().PadRight(33))|" -ForegroundColor Cyan
            Write-Host "+================================================================+" -ForegroundColor Cyan
            Write-Host ""

            # Step 1: Sync main branch
            Write-Log "Step 1/3: Syncing main branch..." -Level INFO
            if (-not (Sync-MainBranch)) {
                Write-Log "Failed to sync main. Retrying in $PollInterval seconds..." -Level ERROR
                Start-Sleep -Seconds $PollInterval
                continue
            }

            # Step 2: Spawn Claude window
            Write-Log "Step 2/3: Spawning Claude Code window..." -Level INFO
            $claudeProcess = Start-ClaudeAutodevWindow

            if ($null -eq $claudeProcess -and -not $DryRun) {
                Write-Log "Failed to spawn Claude window. Retrying in $PollInterval seconds..." -Level ERROR
                Start-Sleep -Seconds $PollInterval
                continue
            }

            # Step 3: Wait for Claude to complete
            Write-Log "Step 3/3: Waiting for Claude to complete..." -Level INFO
            $success = Wait-ForClaudeCompletion -Process $claudeProcess

            if ($success) {
                Write-Host ""
                Write-Host "+================================================================+" -ForegroundColor Green
                Write-Host "|  CYCLE #$($script:CycleCount.ToString().PadRight(3)) COMPLETE                                        |" -ForegroundColor Green
                Write-Host "+================================================================+" -ForegroundColor Green
                Write-Host "|  Check your phone for PR notification                          |" -ForegroundColor Green
                Write-Host "|  Review tests in GitHub                                        |" -ForegroundColor Green
                Write-Host "|  Manually merge when ready                                     |" -ForegroundColor Green
                Write-Host "|                                                                |" -ForegroundColor Green
                Write-Host "|  New cycle will start automatically after merge                |" -ForegroundColor Green
                Write-Host "+================================================================+" -ForegroundColor Green
                Write-Host ""
            }
            else {
                Write-Log "Cycle #$($script:CycleCount) had issues. Continuing monitoring..." -Level WARNING
            }

            # Reset cycle trigger
            $runCycle = $false

            # Update baseline SHA after cycle
            $script:LastKnownMainSha = Get-MainBranchSha
        }

        # Poll for new merges
        $checkTime = Get-Date -Format "HH:mm:ss"
        Write-Host "`r  [$checkTime] Monitoring main branch for merges... (Ctrl+C to stop)     " -NoNewline -ForegroundColor Gray

        if (Test-NewMergeDetected) {
            Write-Host ""  # New line after detecting merge
            $runCycle = $true
        }
        else {
            Start-Sleep -Seconds $PollInterval
        }
    }

    # Summary
    $elapsed = (Get-Date) - $script:StartTime
    Write-Host ""
    Write-Host "+================================================================+" -ForegroundColor Cyan
    Write-Host "|  AUTODEV LOOP FINISHED                                         |" -ForegroundColor Cyan
    Write-Host "+================================================================+" -ForegroundColor Cyan
    Write-Host "|  Total cycles: $($script:CycleCount.ToString().PadRight(49))|" -ForegroundColor Cyan
    Write-Host "|  Total time: $("{0:hh\:mm\:ss}" -f $elapsed)                                            |" -ForegroundColor Cyan
    Write-Host "|  Log file: See console output above                            |" -ForegroundColor Cyan
    Write-Host "+================================================================+" -ForegroundColor Cyan

    Write-Log "Development loop completed. Total cycles: $($script:CycleCount)" -Level SUCCESS
}

#endregion

# Start the loop
Start-DevLoop
