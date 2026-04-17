# url-md installer (Windows PowerShell)
# Usage:
#   irm https://raw.githubusercontent.com/Bwkyd/url-md/main/install.ps1 | iex
#   $v = 'v0.1.1'; irm https://raw.githubusercontent.com/Bwkyd/url-md/main/install.ps1 | iex
#
# Env:
#   $env:URL_MD_INSTALL  override install dir (default: $HOME\.url-md)

$ErrorActionPreference = 'Stop'

$Repo = 'Bwkyd/url-md'
$Version = if ($v) { $v } else { 'latest' }
$InstallDir = if ($env:URL_MD_INSTALL) { $env:URL_MD_INSTALL } else { "$HOME\.url-md" }
$BinDir = "$InstallDir\bin"

# ----- platform detection -----
$Arch = switch ($env:PROCESSOR_ARCHITECTURE) {
    'AMD64' { 'x86_64-pc-windows-msvc' }
    'ARM64' { Write-Error "Windows arm64 prebuilt binary not available yet. Use 'cargo install --git https://github.com/$Repo url-md --locked'." }
    default { Write-Error "Unsupported architecture: $env:PROCESSOR_ARCHITECTURE" }
}

# ----- resolve download URL -----
$Url = if ($Version -eq 'latest') {
    "https://github.com/$Repo/releases/latest/download/url-md-$Arch.zip"
} else {
    "https://github.com/$Repo/releases/download/$Version/url-md-$Arch.zip"
}

# ----- download + extract -----
$null = New-Item -ItemType Directory -Force -Path $BinDir
$Tmp = New-TemporaryFile
Remove-Item $Tmp
$Tmp = "$($Tmp.FullName).d"
$null = New-Item -ItemType Directory -Force -Path $Tmp

try {
    Write-Host "↓ $Url"
    Invoke-WebRequest -Uri $Url -OutFile "$Tmp\url-md.zip" -UseBasicParsing
    Expand-Archive -Path "$Tmp\url-md.zip" -DestinationPath $Tmp -Force

    $Binary = Get-ChildItem -Path $Tmp -Recurse -Filter 'url-md.exe' | Select-Object -First 1
    if (-not $Binary) { Write-Error "url-md.exe not found in archive" }

    Copy-Item -Path $Binary.FullName -Destination "$BinDir\url-md.exe" -Force
} finally {
    Remove-Item -Recurse -Force $Tmp -ErrorAction SilentlyContinue
}

# ----- output -----
$InstalledVersion = (& "$BinDir\url-md.exe" --version 2>$null)
Write-Host ""
Write-Host "✓ Installed: $BinDir\url-md.exe ($InstalledVersion)"
Write-Host ""

# ----- PATH hint -----
$UserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if ($UserPath -split ';' -contains $BinDir) {
    Write-Host "  $BinDir is already on PATH. Run 'url-md --help' to get started (open a new terminal)."
} else {
    Write-Host "  Add to User PATH (one-liner, then reopen terminal):"
    Write-Host ""
    Write-Host "    [Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path','User') + ';$BinDir', 'User')"
}
