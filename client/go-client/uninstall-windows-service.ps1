param(
  [string]$BinaryPath = "$PSScriptRoot\auroraops-agent.exe"
)

if (!(Test-Path $BinaryPath)) {
  Write-Error "Binary not found: $BinaryPath"
  exit 1
}

& $BinaryPath --service uninstall
