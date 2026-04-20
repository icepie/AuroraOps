param(
  [string]$BinaryPath = "$PSScriptRoot\auroraops-agent.exe",
  [string]$ConfigPath = "C:\ProgramData\AuroraOps\agent-config.json",
  [string]$Server = "127.0.0.1:8000",
  [string]$Name = "windows-node-01",
  [int]$Port = 18765
)

if (!(Test-Path $BinaryPath)) {
  Write-Error "Binary not found: $BinaryPath"
  exit 1
}

& $BinaryPath --service install --config $ConfigPath --port $Port --server $Server --name $Name
