# PowerShell script to install storm-cli completions

param(
    [string]$Shell = ""
)

$ErrorActionPreference = "Stop"

Write-Host "Storm CLI Completion Installer" -ForegroundColor Cyan
Write-Host "==============================" -ForegroundColor Cyan
Write-Host

# Check if storm-cli is available
try {
    $null = Get-Command storm-cli -ErrorAction Stop
} catch {
    Write-Host "Error: storm-cli not found in PATH" -ForegroundColor Red
    Write-Host "Please ensure storm-cli is installed and in your PATH"
    exit 1
}

function Install-BashCompletions {
    Write-Host "Installing Bash completions..." -ForegroundColor Yellow

    $completionDir = "$HOME\.bash_completion.d"
    if (-not (Test-Path $completionDir)) {
        New-Item -ItemType Directory -Path $completionDir -Force | Out-Null
    }

    $completionFile = Join-Path $completionDir "storm-cli.bash"

    Write-Host "Generating completions..." -ForegroundColor Green
    & storm-cli completion bash | Out-File -FilePath $completionFile -Encoding utf8

    Write-Host "✓ Installed to: $completionFile" -ForegroundColor Green
    Write-Host
    Write-Host "Add to your ~/.bashrc:" -ForegroundColor Yellow
    Write-Host "  source $completionFile" -ForegroundColor Yellow
}

function Install-PowerShellCompletions {
    Write-Host "Installing PowerShell completions..." -ForegroundColor Yellow

    $profileDir = Split-Path $PROFILE -Parent
    if (-not (Test-Path $profileDir)) {
        New-Item -ItemType Directory -Path $profileDir -Force | Out-Null
    }

    $completionFile = Join-Path $profileDir "storm-cli-completion.ps1"

    Write-Host "Generating completions..." -ForegroundColor Green
    & storm-cli completion powershell | Out-File -FilePath $completionFile -Encoding utf8

    Write-Host "✓ Installed to: $completionFile" -ForegroundColor Green
    Write-Host

    # Check if already in profile
    if (Test-Path $PROFILE) {
        $profileContent = Get-Content $PROFILE -Raw
        if ($profileContent -notmatch "storm-cli-completion\.ps1") {
            Write-Host "Add to your PowerShell profile ($PROFILE):" -ForegroundColor Yellow
            Write-Host "  . `"$completionFile`"" -ForegroundColor Yellow
        } else {
            Write-Host "Completion already registered in profile" -ForegroundColor Green
        }
    } else {
        Write-Host "Creating PowerShell profile and adding completion..." -ForegroundColor Yellow
        ". `"$completionFile`"" | Out-File -FilePath $PROFILE -Encoding utf8
        Write-Host "✓ Added to profile" -ForegroundColor Green
    }
}

# Main logic
if ($Shell) {
    switch ($Shell.ToLower()) {
        "bash" { Install-BashCompletions }
        "powershell" { Install-PowerShellCompletions }
        default {
            Write-Host "Unknown shell: $Shell" -ForegroundColor Red
            Write-Host "Supported shells: bash, powershell"
            exit 1
        }
    }
} else {
    # Default to PowerShell on Windows
    Install-PowerShellCompletions
}

Write-Host
Write-Host "Installation complete!" -ForegroundColor Green
Write-Host "Restart your shell or reload your profile to use completions." -ForegroundColor Yellow
