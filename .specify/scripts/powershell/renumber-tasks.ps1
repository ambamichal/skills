# Renumber tasks in tasks.md to match GitHub Issue numbers
# T001 -> T003, T002 -> T004, ..., T260 -> T262

param(
    [switch]$DryRun,
    [switch]$Json
)

$tasksFile = "specs\1-instruo-mvp-platform\tasks.md"

if (-not (Test-Path $tasksFile)) {
    Write-Error "Tasks file not found: $tasksFile"
    exit 1
}

Write-Host "Renumbering tasks in $tasksFile..." -ForegroundColor Yellow
Write-Host ""

# Read file content
$content = Get-Content $tasksFile -Raw

# Count original task references
$originalCount = ([regex]::Matches($content, '\bT\d{3}\b')).Count
Write-Host "Found $originalCount task references" -ForegroundColor Cyan

# Renumber in reverse order (260 down to 1) to avoid double-replacements
# T260 -> T262, T259 -> T261, ..., T001 -> T003
$replacements = 0

for ($i = 260; $i -ge 1; $i--) {
    $oldId = "T{0:D3}" -f $i
    $newId = "T{0:D3}" -f ($i + 2)

    # Replace all occurrences with word boundaries
    $pattern = "\b$oldId\b"
    $matches = ([regex]::Matches($content, $pattern)).Count

    if ($matches -gt 0) {
        $content = $content -replace $pattern, $newId
        $replacements += $matches

        if (-not $DryRun) {
            Write-Host "  $oldId -> $newId ($matches occurrences)" -ForegroundColor Gray
        }
    }
}

Write-Host ""
Write-Host "Total replacements: $replacements" -ForegroundColor Green

if ($DryRun) {
    Write-Host ""
    Write-Host "DRY RUN - No changes made" -ForegroundColor Yellow
    Write-Host "Run without -DryRun to apply changes" -ForegroundColor Yellow
} else {
    # Write updated content
    $content | Set-Content $tasksFile -NoNewline

    # Verify
    $newContent = Get-Content $tasksFile -Raw
    $newCount = ([regex]::Matches($newContent, '\bT\d{3}\b')).Count

    Write-Host ""
    Write-Host "Verification:" -ForegroundColor Cyan
    Write-Host "  Original references: $originalCount" -ForegroundColor White
    Write-Host "  New references: $newCount" -ForegroundColor White
    Write-Host "  Expected: $originalCount (same count)" -ForegroundColor White

    if ($newCount -eq $originalCount) {
        Write-Host ""
        Write-Host "SUCCESS: Renumbering complete!" -ForegroundColor Green
        Write-Host "Backup saved to: $tasksFile.backup" -ForegroundColor Gray
    } else {
        Write-Host ""
        Write-Host "WARNING: Reference count changed!" -ForegroundColor Yellow
        Write-Host "Please review the changes manually" -ForegroundColor Yellow
    }
}

if ($Json) {
    @{
        originalCount = $originalCount
        newCount = if ($DryRun) { $originalCount } else { $newCount }
        replacements = $replacements
        dryRun = $DryRun.IsPresent
    } | ConvertTo-Json
}

exit 0
