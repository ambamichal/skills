# Update GitHub issue titles to match renumbered tasks
# [T001] -> [T003], [T002] -> [T004], etc.

param(
    [switch]$DryRun,
    [switch]$Json
)

Write-Host "Updating GitHub issue titles..." -ForegroundColor Yellow
Write-Host ""

# Get all issues
$issuesJson = & gh issue list --state all --limit 300 --json number,title 2>&1

if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to fetch issues: $issuesJson"
    exit 1
}

$issues = $issuesJson | ConvertFrom-Json

$stats = @{
    Total = $issues.Count
    Updated = 0
    Failed = 0
    Skipped = 0
}

foreach ($issue in $issues) {
    $number = $issue.number
    $oldTitle = $issue.title

    # Check if title contains [TXXX] pattern
    if ($oldTitle -match '^\[T(\d{3})\]\s*(.+)$') {
        $oldTaskNum = [int]$Matches[1]
        $description = $Matches[2]

        # Calculate new task number (old + 2)
        $newTaskNum = $oldTaskNum + 2
        $newTaskId = "T{0:D3}" -f $newTaskNum
        $newTitle = "[$newTaskId] $description"

        if ($oldTitle -eq $newTitle) {
            Write-Host "  Skip #$number : Already correct" -ForegroundColor Gray
            $stats.Skipped++
            continue
        }

        if ($DryRun) {
            Write-Host "  Would update #$number :" -ForegroundColor Cyan
            Write-Host "    Old: $oldTitle" -ForegroundColor Gray
            Write-Host "    New: $newTitle" -ForegroundColor Green
            $stats.Updated++
        } else {
            Write-Host "  Updating #$number : [T$('{0:D3}' -f $oldTaskNum)] -> [$newTaskId]" -ForegroundColor Yellow

            $result = & gh issue edit $number --title $newTitle 2>&1

            if ($LASTEXITCODE -eq 0) {
                Write-Host "    Success" -ForegroundColor Green
                $stats.Updated++
            } else {
                Write-Host "    Failed: $result" -ForegroundColor Red
                $stats.Failed++
            }

            # Small delay to avoid rate limiting
            Start-Sleep -Milliseconds 100
        }
    } else {
        Write-Host "  Skip #$number : No task ID in title" -ForegroundColor Gray
        $stats.Skipped++
    }
}

Write-Host ""
Write-Host "Summary:" -ForegroundColor Cyan
Write-Host "  Total issues: $($stats.Total)" -ForegroundColor White
Write-Host "  Updated: $($stats.Updated)" -ForegroundColor Green
Write-Host "  Failed: $($stats.Failed)" -ForegroundColor Red
Write-Host "  Skipped: $($stats.Skipped)" -ForegroundColor Gray

if ($DryRun) {
    Write-Host ""
    Write-Host "DRY RUN - No changes made" -ForegroundColor Yellow
    Write-Host "Run without -DryRun to apply changes" -ForegroundColor Yellow
}

if ($Json) {
    $stats | ConvertTo-Json
}

exit 0
