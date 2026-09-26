# Create all required GitHub labels
# This script creates labels used for issue organization

param(
    [switch]$DryRun,
    [switch]$Json
)

$labels = @(
    # Phase labels (green shades)
    @{ name = "phase-1"; color = "0E8A16"; description = "Phase 1: Setup" }
    @{ name = "phase-2"; color = "1A9E2B"; description = "Phase 2: Foundational" }
    @{ name = "phase-3"; color = "26B340"; description = "Phase 3: US1 - School Admin Onboarding" }
    @{ name = "phase-4"; color = "32C755"; description = "Phase 4: US2 - Instructor Management" }
    @{ name = "phase-5"; color = "3EDC6A"; description = "Phase 5: US3 - Client Registration" }
    @{ name = "phase-6"; color = "4AF07F"; description = "Phase 6: US4 - Lesson Booking" }
    @{ name = "phase-7"; color = "56FF94"; description = "Phase 7: US6 - Online Payments" }
    @{ name = "phase-8"; color = "62FFA9"; description = "Phase 8: US7 - Instructor Financials" }
    @{ name = "phase-9"; color = "6EFFBE"; description = "Phase 9: US5 - Interactive Calendar" }
    @{ name = "phase-10"; color = "7AFFD3"; description = "Phase 10: US10 - Admin Panel" }
    @{ name = "phase-11"; color = "86FFE8"; description = "Phase 11: US8 - Client Progress" }
    @{ name = "phase-12"; color = "92FFFD"; description = "Phase 12: US9 - Notifications" }
    @{ name = "phase-13"; color = "9EFFFF"; description = "Phase 13: Polish & Cross-Cutting" }

    # Priority labels (red to yellow)
    @{ name = "p1"; color = "D73A4A"; description = "Priority 1: Critical - MVP blocker" }
    @{ name = "p2"; color = "FBCA04"; description = "Priority 2: Enhanced - Important feature" }
    @{ name = "p3"; color = "FEF2C0"; description = "Priority 3: Polish - Nice to have" }

    # User Story labels (blue shades)
    @{ name = "us1"; color = "0052CC"; description = "US1: School Admin Onboarding & Dashboard" }
    @{ name = "us2"; color = "0066FF"; description = "US2: Instructor Management" }
    @{ name = "us3"; color = "1A7AFF"; description = "US3: Client Registration & Profile" }
    @{ name = "us4"; color = "338EFF"; description = "US4: Lesson Booking System" }
    @{ name = "us5"; color = "4DA2FF"; description = "US5: Interactive Calendar" }
    @{ name = "us6"; color = "66B6FF"; description = "US6: Online Payment Processing" }
    @{ name = "us7"; color = "80CAFF"; description = "US7: Instructor Financials & Settlements" }
    @{ name = "us8"; color = "99DEFF"; description = "US8: Client Lesson History & Progress" }
    @{ name = "us9"; color = "B3F2FF"; description = "US9: SMS & Email Notifications" }
    @{ name = "us10"; color = "CCFFFF"; description = "US10: Admin Panel for Daily Operations" }

    # Technical labels (various colors)
    @{ name = "backend"; color = "5319E7"; description = "Backend: Django, API, database" }
    @{ name = "frontend"; color = "1D76DB"; description = "Frontend: Next.js, React, UI" }
    @{ name = "api"; color = "F9D0C4"; description = "API: REST endpoints, contracts" }
    @{ name = "ui"; color = "C2E0C6"; description = "UI: Components, styling, UX" }
    @{ name = "database"; color = "006B75"; description = "Database: Models, migrations, queries" }
    @{ name = "devops"; color = "BFD4F2"; description = "DevOps: Docker, CI/CD, deployment" }
)

function Create-Label {
    param(
        [string]$Name,
        [string]$Color,
        [string]$Description,
        [switch]$DryRun
    )

    if ($DryRun) {
        Write-Host "Would create label: $Name ($Color) - $Description" -ForegroundColor Cyan
        return $true
    }

    $result = & gh label create $Name --color $Color --description $Description 2>&1
    $exitCode = $LASTEXITCODE

    if ($exitCode -eq 0) {
        Write-Host "Created: $Name" -ForegroundColor Green
        return $true
    } elseif ($result -like "*already exists*") {
        Write-Host "Exists: $Name" -ForegroundColor Gray
        return $true
    } else {
        Write-Host "Failed: $Name - $result" -ForegroundColor Red
        return $false
    }
}

# Main execution
$ghVersion = & gh --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Error "GitHub CLI (gh) is not installed"
    exit 1
}

$authStatus = & gh auth status 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Error "Not authenticated with GitHub CLI"
    exit 1
}

Write-Host "Creating GitHub labels..." -ForegroundColor Yellow
Write-Host ""

$stats = @{
    Total = $labels.Count
    Created = 0
    Failed = 0
}

foreach ($label in $labels) {
    $success = Create-Label -Name $label.name -Color $label.color -Description $label.description -DryRun:$DryRun

    if ($success) {
        $stats.Created++
    } else {
        $stats.Failed++
    }

    Start-Sleep -Milliseconds 50
}

Write-Host ""
Write-Host "Summary:" -ForegroundColor Cyan
Write-Host "  Total labels: $($stats.Total)" -ForegroundColor White
Write-Host "  Created/Exists: $($stats.Created)" -ForegroundColor Green
Write-Host "  Failed: $($stats.Failed)" -ForegroundColor Red

if ($Json) {
    $stats | ConvertTo-Json
}

exit 0
