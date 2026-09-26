# Add labels to existing GitHub issues
# This script reads issue metadata and adds appropriate labels

param(
    [switch]$DryRun,
    [switch]$Json
)

. "$PSScriptRoot\common.ps1"

function Extract-Labels {
    param(
        [string]$Body,
        [string]$Title
    )

    $labels = @()

    # Extract Phase from body
    if ($Body -match '\*\*Phase\*\*:\s*(\d+)') {
        $phaseNum = $Matches[1]
        $labels += "phase-$phaseNum"
    }

    # Extract Priority from body
    if ($Body -match '\*\*Priority\*\*:\s*(P\d+)') {
        $priority = $Matches[1].ToLower()
        $labels += $priority
    }

    # Extract Story from body or title
    if ($Body -match '\[US(\d+)\]' -or $Title -match '\[US(\d+)\]') {
        $storyNum = if ($Matches[1]) { $Matches[1] } else { $Matches[1] }
        $labels += "us$storyNum"
    }

    # Determine technical category from description
    $fullText = "$Title $Body".ToLower()

    if ($fullText -match '(backend|django|api|model|serializer|view|celery|database|migration|postgres)') {
        $labels += "backend"
    }
    if ($fullText -match '(frontend|next\.?js|react|component|ui|tailwind|shadcn)') {
        $labels += "frontend"
    }
    if ($fullText -match '(endpoint|api|rest|graphql|contract)') {
        $labels += "api"
    }
    if ($fullText -match '(ui|button|form|modal|calendar|dashboard|interface)') {
        $labels += "ui"
    }
    if ($fullText -match '(database|migration|schema|postgres|sql|model)') {
        $labels += "database"
    }
    if ($fullText -match '(docker|ci/cd|pipeline|deployment|github\s+actions)') {
        $labels += "devops"
    }

    return $labels | Select-Object -Unique
}

function Add-Labels-To-Issue {
    param(
        [int]$IssueNumber,
        [string[]]$Labels,
        [switch]$DryRun
    )

    if ($Labels.Count -eq 0) {
        return $true
    }

    if ($DryRun) {
        Write-Host "Would add labels to #$IssueNumber : $($Labels -join ', ')" -ForegroundColor Cyan
        return $true
    }

    try {
        $labelArgs = $Labels | ForEach-Object { "--add-label", $_ }
        $result = & gh issue edit $IssueNumber @labelArgs 2>&1

        if ($LASTEXITCODE -eq 0) {
            Write-Host "✓ Updated #$IssueNumber : $($Labels -join ', ')" -ForegroundColor Green
            return $true
        } else {
            Write-Host "✗ Failed #$IssueNumber : $result" -ForegroundColor Red
            return $false
        }
    } catch {
        Write-Host "✗ Error updating #$IssueNumber : $_" -ForegroundColor Red
        return $false
    }
}

# Main execution
try {
    # Check if gh CLI is installed
    $ghVersion = & gh --version 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Error "GitHub CLI (gh) is not installed. Install from: https://cli.github.com/"
        exit 1
    }

    # Check if authenticated
    $authStatus = & gh auth status 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Not authenticated with GitHub CLI. Run: gh auth login"
        exit 1
    }

    Write-Host "Fetching all open issues..." -ForegroundColor Yellow

    # Get all open issues
    $issuesJson = & gh issue list --limit 1000 --state open --json number,title,body 2>&1

    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to fetch issues: $issuesJson"
        exit 1
    }

    $issues = $issuesJson | ConvertFrom-Json

    Write-Host "Found $($issues.Count) open issues" -ForegroundColor Yellow
    Write-Host ""

    $stats = @{
        Total = $issues.Count
        Updated = 0
        Failed = 0
        Skipped = 0
    }

    foreach ($issue in $issues) {
        $labels = Extract-Labels -Body $issue.body -Title $issue.title

        if ($labels.Count -eq 0) {
            Write-Host "⊘ Skipping #$($issue.number) (no labels to add)" -ForegroundColor Gray
            $stats.Skipped++
            continue
        }

        $success = Add-Labels-To-Issue -IssueNumber $issue.number -Labels $labels -DryRun:$DryRun

        if ($success) {
            $stats.Updated++
        } else {
            $stats.Failed++
        }

        # Rate limiting: small delay between requests
        Start-Sleep -Milliseconds 100
    }

    Write-Host ""
    Write-Host "═══════════════════════════════════════" -ForegroundColor Cyan
    Write-Host "Summary:" -ForegroundColor Cyan
    Write-Host "  Total issues: $($stats.Total)" -ForegroundColor White
    Write-Host "  Updated: $($stats.Updated)" -ForegroundColor Green
    Write-Host "  Failed: $($stats.Failed)" -ForegroundColor Red
    Write-Host "  Skipped: $($stats.Skipped)" -ForegroundColor Gray
    Write-Host "═══════════════════════════════════════" -ForegroundColor Cyan

    if ($Json) {
        $stats | ConvertTo-Json
    }

    exit 0

} catch {
    Write-Error "Script failed: $_"
    if ($Json) {
        @{ error = $_.Exception.Message } | ConvertTo-Json
    }
    exit 1
}
