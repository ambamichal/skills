# Create GitHub Issues from tasks.md
# This script parses tasks.md and creates individual GitHub issues for each task

param(
    [switch]$DryRun,
    [switch]$Json
)

. "$PSScriptRoot\common.ps1"

function Parse-TaskLine {
    param([string]$Line)

    if ($Line -match '^\s*-\s*\[\s*\]\s*T(\d+)\s*(.*)$') {
        $taskId = "T" + $Matches[1]
        $rest = $Matches[2].Trim()

        $isParallel = $false
        $story = $null
        $description = $rest

        if ($rest -match '^\[P\]\s*(.*)$') {
            $isParallel = $true
            $rest = $Matches[1].Trim()
        }

        if ($rest -match '^\[(US\d+)\]\s*(.*)$') {
            $story = $Matches[1]
            $description = $Matches[2].Trim()
        } else {
            $description = $rest
        }

        return @{
            Id = $taskId
            IsParallel = $isParallel
            Story = $story
            Description = $description
        }
    }

    return $null
}

function Get-PhaseInfo {
    param([string]$Line)

    if ($Line -match '^##\s+Phase\s+(\d+):\s*(.+?)(\s*\(Priority:\s*(P\d+)\))?$') {
        return @{
            PhaseNumber = [int]$Matches[1]
            PhaseTitle = $Matches[2].Trim()
            Priority = if ($Matches[4]) { $Matches[4] } else { $null }
        }
    }

    return $null
}

function Create-GitHubIssue {
    param(
        [string]$Title,
        [string]$Body,
        [string[]]$Labels,
        [switch]$DryRun
    )

    if ($DryRun) {
        Write-Host "Would create issue: $Title" -ForegroundColor Cyan
        Write-Host "  Labels: $($Labels -join ', ')" -ForegroundColor Gray
        return $true
    }

    try {
        $result = & gh issue create --title $Title --body $Body 2>&1

        if ($LASTEXITCODE -eq 0) {
            Write-Host "✓ Created: $Title" -ForegroundColor Green
            return $true
        } else {
            Write-Host "✗ Failed: $Title - $result" -ForegroundColor Red
            return $false
        }
    } catch {
        Write-Host "✗ Error creating issue: $Title - $_" -ForegroundColor Red
        return $false
    }
}

# Main execution
try {
    $ghVersion = & gh --version 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Error "GitHub CLI (gh) is not installed. Install from: https://cli.github.com/"
        exit 1
    }

    $authStatus = & gh auth status 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Not authenticated with GitHub CLI. Run: gh auth login"
        exit 1
    }

    $tasksPath = Join-Path $PSScriptRoot "..\..\..\specs\1-instruo-mvp-platform\tasks.md"

    if (-not (Test-Path $tasksPath)) {
        Write-Error "Tasks file not found: $tasksPath"
        exit 1
    }

    Write-Host "Parsing tasks from: $tasksPath" -ForegroundColor Yellow

    $lines = Get-Content $tasksPath
    $currentPhase = $null
    $tasks = @()
    $phaseGoal = ""
    $phaseTest = ""

    foreach ($line in $lines) {
        $phaseInfo = Get-PhaseInfo $line
        if ($phaseInfo) {
            $currentPhase = $phaseInfo
            $phaseGoal = ""
            $phaseTest = ""
            continue
        }

        if ($line -match '^\*\*Goal\*\*:\s*(.+)$') {
            $phaseGoal = $Matches[1]
            continue
        }

        if ($line -match '^\*\*Independent Test\*\*:\s*(.+)$') {
            $phaseTest = $Matches[1]
            continue
        }

        $task = Parse-TaskLine $line
        if ($task -and $currentPhase) {
            $task.Phase = $currentPhase.PhaseNumber
            $task.PhaseTitle = $currentPhase.PhaseTitle
            $task.Priority = $currentPhase.Priority
            $task.PhaseGoal = $phaseGoal
            $task.PhaseTest = $phaseTest
            $tasks += $task
        }
    }

    Write-Host "Found $($tasks.Count) tasks to create" -ForegroundColor Yellow

    if ($DryRun) {
        Write-Host "`nDRY RUN MODE - No issues will be created`n" -ForegroundColor Magenta
    }

    $created = 0
    $failed = 0

    foreach ($task in $tasks) {
        $title = "[$($task.Id)] $($task.Description)"

        $bodyParts = @()
        $bodyParts += "## Task: $($task.Id)"
        $bodyParts += ""
        $bodyParts += "**Phase**: $($task.Phase) - $($task.PhaseTitle)"

        if ($task.Priority) {
            $bodyParts += "**Priority**: $($task.Priority)"
        }

        if ($task.Story) {
            $bodyParts += "**User Story**: $($task.Story)"
        }

        if ($task.IsParallel) {
            $bodyParts += "**Parallelizable**: Yes (can run in parallel with other [P] tasks)"
        }

        $bodyParts += ""
        $bodyParts += "### Description"
        $bodyParts += $task.Description

        if ($task.PhaseGoal) {
            $bodyParts += ""
            $bodyParts += "### Phase Goal"
            $bodyParts += $task.PhaseGoal
        }

        if ($task.PhaseTest) {
            $bodyParts += ""
            $bodyParts += "### Acceptance Criteria"
            $bodyParts += $task.PhaseTest
        }

        $bodyParts += ""
        $bodyParts += "---"
        $bodyParts += "*Generated from tasks.md - Phase $($task.Phase)*"

        $body = $bodyParts -join "`n"

        $labels = @("phase-$($task.Phase)")

        if ($task.Priority) {
            $labels += $task.Priority.ToLower()
        }

        if ($task.Story) {
            $labels += $task.Story.ToLower()
        }

        if ($task.IsParallel) {
            $labels += "parallel"
        }

        if ($task.Description -match 'backend/') {
            $labels += "backend"
        }
        if ($task.Description -match 'frontend/') {
            $labels += "frontend"
        }
        if ($task.Description -match 'model|models\.py') {
            $labels += "database"
        }
        if ($task.Description -match 'api/|endpoint|serializer') {
            $labels += "api"
        }
        if ($task.Description -match 'component|page\.tsx') {
            $labels += "ui"
        }
        if ($task.Description -match 'docker|deployment|ci/cd') {
            $labels += "devops"
        }

        $success = Create-GitHubIssue -Title $title -Body $body -Labels $labels -DryRun:$DryRun

        if ($success) {
            $created++
        } else {
            $failed++
        }

        if (-not $DryRun) {
            Start-Sleep -Milliseconds 500
        }
    }

    Write-Host "`n========================================" -ForegroundColor Cyan
    Write-Host "Summary:" -ForegroundColor Yellow
    Write-Host "  Total tasks: $($tasks.Count)" -ForegroundColor White
    Write-Host "  Created: $created" -ForegroundColor Green
    if ($failed -gt 0) {
        Write-Host "  Failed: $failed" -ForegroundColor Red
    }
    Write-Host "========================================`n" -ForegroundColor Cyan

    if ($Json) {
        @{
            total = $tasks.Count
            created = $created
            failed = $failed
            dryRun = $DryRun.IsPresent
        } | ConvertTo-Json
    }

} catch {
    Write-Error "Error: $_"
    if ($Json) {
        @{
            error = $_.Exception.Message
        } | ConvertTo-Json
    }
    exit 1
}
