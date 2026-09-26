<#
.SYNOPSIS
    Automated Development Cycle Helper Script for AutoDev

.DESCRIPTION
    This PowerShell script provides helper functions for the automated development workflow.
    It can be used standalone or called by Claude Code's /autodev command.

.PARAMETER Action
    The workflow action to perform:
    - CheckNext: Find next sequential issue
    - CreateBranch: Create and checkout feature branch
    - RunTests: Execute all tests (backend + frontend)
    - CreatePR: Create pull request
    - Cleanup: Return to main and cleanup

.PARAMETER IssueNumber
    GitHub issue number to work on (optional, defaults to next sequential)

.PARAMETER BranchName
    Feature branch name (auto-generated if not provided)

.PARAMETER DryRun
    Show what would be done without executing

.EXAMPLE
    .\Invoke-AutoDev.ps1 -Action CheckNext
    Find the next sequential GitHub issue

.EXAMPLE
    .\Invoke-AutoDev.ps1 -Action CreateBranch -IssueNumber 7
    Create feature branch for issue #7

.EXAMPLE
    .\Invoke-AutoDev.ps1 -Action RunTests
    Execute all backend and frontend tests

.EXAMPLE
    .\Invoke-AutoDev.ps1 -Action CreatePR -IssueNumber 7
    Create pull request for issue #7

.NOTES
    Version: 1.0.0
    Author: AutoDev Framework
    Requires: Git, GitHub CLI (gh), Python 3.11+, Node.js 20+
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('CheckNext', 'CreateBranch', 'RunTests', 'CreatePR', 'Cleanup', 'FullCycle')]
    [string]$Action,

    [Parameter(Mandatory = $false)]
    [int]$IssueNumber,

    [Parameter(Mandatory = $false)]
    [string]$BranchName,

    [Parameter(Mandatory = $false)]
    [switch]$DryRun,

    [Parameter(Mandatory = $false)]
    [switch]$AutoPush,

    [Parameter(Mandatory = $false)]
    [switch]$AutoPR,

    [Parameter(Mandatory = $false)]
    [switch]$Verbose
)

# Set error action preference
$ErrorActionPreference = "Stop"

# Script configuration
$script:Config = @{
    RootDir         = Get-Location
    BackendDir      = Join-Path (Get-Location) "backend"
    FrontendDir     = Join-Path (Get-Location) "frontend"
    SpecsDir        = Join-Path (Get-Location) "specs"
    ConfigFile      = Join-Path (Get-Location) ".claude/config/autodev-workflow.md"
    MaxRetries      = 4
    RetryDelays     = @(2, 4, 8, 16)  # Exponential backoff in seconds
}

#region Helper Functions

function Write-Step {
    param([string]$Message)
    Write-Host "`n🔹 $Message" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "✅ $Message" -ForegroundColor Green
}

function Write-Error {
    param([string]$Message)
    Write-Host "❌ $Message" -ForegroundColor Red
}

function Write-Warning {
    param([string]$Message)
    Write-Host "⚠️  $Message" -ForegroundColor Yellow
}

function Invoke-GitCommand {
    param(
        [string]$Command,
        [int]$Retries = 1
    )

    for ($i = 0; $i -lt $Retries; $i++) {
        try {
            if ($DryRun) {
                Write-Host "[DRY RUN] git $Command" -ForegroundColor Yellow
                return $true
            }

            $output = git $Command.Split(' ') 2>&1
            if ($LASTEXITCODE -ne 0) {
                throw "Git command failed: $output"
            }
            return $output
        }
        catch {
            if ($i -eq $Retries - 1) {
                throw
            }
            $delay = $script:Config.RetryDelays[$i]
            Write-Warning "Git command failed, retrying in $delay seconds... (Attempt $($i + 1)/$Retries)"
            Start-Sleep -Seconds $delay
        }
    }
}

function Get-NextIssue {
    Write-Step "Finding next sequential GitHub issue..."

    try {
        # Get all open issues sorted by number
        $issues = gh issue list --state open --json number,title --limit 100 | ConvertFrom-Json

        if ($issues.Count -eq 0) {
            Write-Success "No open issues found - all tasks completed! 🎉"
            return $null
        }

        # Get closed issues to find highest completed number
        $closedIssues = gh issue list --state closed --json number --limit 100 | ConvertFrom-Json
        $maxClosed = ($closedIssues | Measure-Object -Property number -Maximum).Maximum

        # Find first open issue after highest closed
        $nextIssue = $issues | Where-Object { $_.number -gt $maxClosed } |
                     Sort-Object number |
                     Select-Object -First 1

        if ($null -eq $nextIssue) {
            # If no issue found, take the first open issue
            $nextIssue = $issues | Sort-Object number | Select-Object -First 1
        }

        Write-Success "Next issue: #$($nextIssue.number) - $($nextIssue.title)"
        return $nextIssue
    }
    catch {
        Write-Error "Failed to fetch GitHub issues: $_"
        throw
    }
}

function New-FeatureBranch {
    param(
        [int]$IssueNumber,
        [string]$BranchName
    )

    Write-Step "Creating feature branch for issue #$IssueNumber..."

    # Ensure we're on main
    Write-Host "Checking out main branch..."
    Invoke-GitCommand "checkout main"

    # Pull latest changes
    Write-Host "Pulling latest changes..."
    Invoke-GitCommand "pull origin main" -Retries 4

    # Create and checkout feature branch
    Write-Host "Creating branch: $BranchName..."
    try {
        Invoke-GitCommand "checkout -b $BranchName"
        Write-Success "Branch created and checked out: $BranchName"
    }
    catch {
        # Branch might already exist
        Write-Warning "Branch might already exist, checking out..."
        Invoke-GitCommand "checkout $BranchName"
        Write-Success "Checked out existing branch: $BranchName"
    }
}

function Invoke-BackendTests {
    Write-Step "Running backend tests..."

    Push-Location $script:Config.BackendDir
    try {
        # Check Django system
        Write-Host "Running Django system checks..."
        if (-not $DryRun) {
            python manage.py check
            if ($LASTEXITCODE -ne 0) { throw "Django check failed" }
        }
        Write-Success "Django system check passed"

        # Run migrations check
        Write-Host "Checking migrations..."
        if (-not $DryRun) {
            python manage.py makemigrations --check --dry-run
            if ($LASTEXITCODE -ne 0) {
                Write-Warning "Pending migrations detected"
            }
        }

        # Run pytest
        Write-Host "Running pytest..."
        if (-not $DryRun) {
            pytest --cov --cov-report=term-missing
            if ($LASTEXITCODE -ne 0) { throw "Pytest failed" }
        }
        Write-Success "All backend tests passed"

        # Run linting
        Write-Host "Running ruff linter..."
        if (-not $DryRun) {
            ruff check .
            if ($LASTEXITCODE -ne 0) { throw "Ruff linting failed" }
        }
        Write-Success "Backend linting passed"

        return $true
    }
    catch {
        Write-Error "Backend tests failed: $_"
        return $false
    }
    finally {
        Pop-Location
    }
}

function Invoke-FrontendTests {
    Write-Step "Running frontend tests..."

    # Check if frontend directory exists and has package.json
    if (-not (Test-Path (Join-Path $script:Config.FrontendDir "package.json"))) {
        Write-Warning "Frontend not initialized yet, skipping tests"
        return $true
    }

    Push-Location $script:Config.FrontendDir
    try {
        # Run build
        Write-Host "Running production build..."
        if (-not $DryRun) {
            npm run build
            if ($LASTEXITCODE -ne 0) { throw "Build failed" }
        }
        Write-Success "Frontend build successful"

        # Run tests
        Write-Host "Running tests..."
        if (-not $DryRun) {
            npm test -- --passWithNoTests
            if ($LASTEXITCODE -ne 0) { throw "Tests failed" }
        }
        Write-Success "All frontend tests passed"

        # Run linting
        Write-Host "Running ESLint..."
        if (-not $DryRun) {
            npm run lint
            if ($LASTEXITCODE -ne 0) { throw "Linting failed" }
        }
        Write-Success "Frontend linting passed"

        # Run type checking
        Write-Host "Running TypeScript type check..."
        if (-not $DryRun) {
            npm run type-check
            if ($LASTEXITCODE -ne 0) { throw "Type check failed" }
        }
        Write-Success "TypeScript type check passed"

        return $true
    }
    catch {
        Write-Error "Frontend tests failed: $_"
        return $false
    }
    finally {
        Pop-Location
    }
}

function Invoke-AllTests {
    $backendPassed = Invoke-BackendTests
    $frontendPassed = Invoke-FrontendTests

    if ($backendPassed -and $frontendPassed) {
        Write-Success "All tests passed! ✅"
        return $true
    }
    else {
        Write-Error "Some tests failed ❌"
        return $false
    }
}

function New-PullRequest {
    param(
        [int]$IssueNumber,
        [string]$IssueTitle
    )

    Write-Step "Creating pull request for issue #$IssueNumber..."

    # Get issue details
    $issue = gh issue view $IssueNumber --json title,body | ConvertFrom-Json

    # Pad issue number
    $paddedNumber = $IssueNumber.ToString("D3")

    # Generate PR title
    $prTitle = "[T$paddedNumber] $($issue.title)"

    # Get list of changed files
    $changedFiles = git diff --name-only main...HEAD

    # Generate PR body
    $prBody = @"
## Summary
Completes Issue #$IssueNumber (Task T$paddedNumber)

$($issue.body)

## Changes
$($changedFiles | ForEach-Object { "- $_" } | Out-String)

## Testing
``````bash
# Backend validation
cd backend
python manage.py check          # ✅ No issues
pytest --cov                    # ✅ All tests pass
ruff check .                    # ✅ No linting errors

# Frontend validation
cd frontend
npm run build                   # ✅ Build successful
npm test                        # ✅ All tests pass
npm run lint                    # ✅ No linting errors
``````

## Constitution Compliance
- ✅ Spec-Driven Development
- ✅ API-First Architecture
- ✅ Multi-Tenancy Discipline
- ✅ Independent User Stories
- ✅ Issue-Driven Development

Closes #$IssueNumber
"@

    if ($DryRun) {
        Write-Host "[DRY RUN] Would create PR:" -ForegroundColor Yellow
        Write-Host "Title: $prTitle"
        Write-Host "Body:`n$prBody"
        return
    }

    # Create PR
    try {
        $prUrl = gh pr create --title $prTitle --body $prBody
        Write-Success "Pull request created: $prUrl"
        return $prUrl
    }
    catch {
        Write-Error "Failed to create pull request: $_"
        throw
    }
}

function Invoke-Cleanup {
    Write-Step "Cleaning up and returning to main branch..."

    # Checkout main
    Invoke-GitCommand "checkout main"

    # Pull latest
    Invoke-GitCommand "pull origin main" -Retries 4

    # Prune deleted branches
    Invoke-GitCommand "fetch --prune"

    Write-Success "Cleanup complete - ready for next issue"
}

#endregion

#region Main Actions

switch ($Action) {
    'CheckNext' {
        $nextIssue = Get-NextIssue
        if ($null -ne $nextIssue) {
            return @{
                Number = $nextIssue.number
                Title  = $nextIssue.title
                Branch = "feature/issue-$($nextIssue.number)-$(($nextIssue.title -replace '[^\w\s-]', '' -replace '\s+', '-').ToLower())"
            }
        }
    }

    'CreateBranch' {
        if (-not $IssueNumber) {
            $nextIssue = Get-NextIssue
            $IssueNumber = $nextIssue.number
        }

        if (-not $BranchName) {
            $issue = gh issue view $IssueNumber --json title | ConvertFrom-Json
            $BranchName = "feature/issue-$IssueNumber-$(($issue.title -replace '[^\w\s-]', '' -replace '\s+', '-').ToLower())"
        }

        New-FeatureBranch -IssueNumber $IssueNumber -BranchName $BranchName
    }

    'RunTests' {
        $success = Invoke-AllTests
        if (-not $success) {
            exit 1
        }
    }

    'CreatePR' {
        if (-not $IssueNumber) {
            Write-Error "IssueNumber parameter required for CreatePR action"
            exit 1
        }

        $issue = gh issue view $IssueNumber --json title | ConvertFrom-Json
        New-PullRequest -IssueNumber $IssueNumber -IssueTitle $issue.title
    }

    'Cleanup' {
        Invoke-Cleanup
    }

    'FullCycle' {
        Write-Host @"
╔════════════════════════════════════════════════════════════╗
║                                                            ║
║        AUTODEV AUTOMATED DEVELOPMENT CYCLE                 ║
║                                                            ║
╚════════════════════════════════════════════════════════════╝
"@ -ForegroundColor Cyan

        # Step 1: Find next issue
        $nextIssue = Get-NextIssue
        if ($null -eq $nextIssue) {
            Write-Success "All issues completed! 🎉"
            return
        }

        $IssueNumber = $nextIssue.number
        $IssueTitle = $nextIssue.title
        $BranchName = "feature/issue-$IssueNumber-$(($IssueTitle -replace '[^\w\s-]', '' -replace '\s+', '-').ToLower())"

        # Step 2: Create branch
        New-FeatureBranch -IssueNumber $IssueNumber -BranchName $BranchName

        # Step 3: Implementation (manual or via Claude)
        Write-Warning "Implementation phase should be handled by Claude Code's master-orchestrator agent"
        Write-Host "Waiting for implementation to complete..."
        Write-Host "Press ENTER when implementation is done and you're ready to test..."
        if (-not $DryRun) {
            Read-Host
        }

        # Step 4: Run tests
        $testsPassed = Invoke-AllTests
        if (-not $testsPassed) {
            Write-Error "Tests failed. Fix issues before creating PR."
            exit 1
        }

        # Step 5: Git operations
        Write-Step "Staging changes..."
        Invoke-GitCommand "add ."

        Write-Step "Committing changes..."
        $commitMsg = "feat: $IssueTitle #$IssueNumber"
        Invoke-GitCommand "commit -m `"$commitMsg`""

        if ($AutoPush -or $DryRun) {
            Write-Step "Pushing to remote..."
            Invoke-GitCommand "push -u origin $BranchName" -Retries 4
        }
        else {
            Write-Host "`nReady to push changes. Run: git push -u origin $BranchName"
        }

        # Step 6: Create PR
        if ($AutoPR -or $DryRun) {
            New-PullRequest -IssueNumber $IssueNumber -IssueTitle $IssueTitle
        }
        else {
            Write-Host "`nReady to create PR. Run: gh pr create"
        }

        # Step 7: Cleanup
        Invoke-Cleanup

        Write-Host @"

╔════════════════════════════════════════════════════════════╗
║                                                            ║
║              DEVELOPMENT CYCLE COMPLETE! ✅                ║
║                                                            ║
║  Issue: #$IssueNumber - $IssueTitle
║                                                            ║
╚════════════════════════════════════════════════════════════╝
"@ -ForegroundColor Green
    }
}

Write-Host ""
