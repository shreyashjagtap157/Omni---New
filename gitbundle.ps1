$ErrorActionPreference = "Stop"

# Change to the Omni directory
$omniDir = "C:\Users\siddh\Downloads\ABC\Omni"
Set-Location $omniDir

Write-Output "=== Step 1: Remove untracked temp file if present ==="
if (Test-Path "gitcommands.ps1") {
    Remove-Item "gitcommands.ps1"
    Write-Output "Removed temp file"
}

Write-Output "=== Step 2: Check git status ==="
git status

Write-Output "=== Step 3: Add all changes (including gitignore) ==="
git add -A

Write-Output "=== Step 4: Commit the gitignore update and all changes ==="
# Check if there are any changes to commit
if (git diff --cached --quiet) {
    Write-Output "No changes to commit"
} else {
    git commit -m "Update gitignore and add proper project exclusions"
    Write-Output "Commit successful"
}

Write-Output "=== Step 5: Push to remote ==="
git push origin main
Write-Output "Push successful"

Write-Output "=== Step 6: Create a complete history bundle ==="
# Get the current date for the bundle filename
$date = Get-Date -Format "yyyyMMdd_HHmmss"
$bundleName = "omni-complete-history_$date.bundle"

# Create a bundle with all branches and tags
# --all includes all branches; --tags includes all tags
# The bundle format v2 is the default newer format
git bundle create "$bundleName" --all --tags

Write-Output "=== Bundle created: $bundleName ==="
Write-Output "=== Bundle info ==="
# Show bundle info
git bundle info "$bundleName" 2>&1 | Write-Output

Write-Output "=== All steps completed successfully! ==="
Write-Output "The bundle can be cloned on another device with:"
Write-Output "git bundle unbundle omni-complete-history_*.bundle"