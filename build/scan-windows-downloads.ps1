param(
    [Parameter(Mandatory)]
    [string[]]$Paths,
    [Parameter(Mandatory)]
    [string]$ReportPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
# The positive control deliberately returns Defender's detection exit code.
$PSNativeCommandUseErrorActionPreference = $false

New-Item -ItemType Directory -Path (Split-Path -Parent $ReportPath) -Force | Out-Null
Set-Content -LiteralPath $ReportPath -Value 'Microsoft Defender custom scan report' -Encoding utf8
function Write-ScanLog([string]$Message) {
    Write-Host $Message
    Add-Content -LiteralPath $ReportPath -Value $Message -Encoding utf8
}

$resolved = @($Paths | ForEach-Object { (Resolve-Path -LiteralPath $_).ProviderPath })
if ($resolved.Count -eq 0) { throw 'No files or directories selected for scanning.' }
Write-ScanLog "UTC: $([DateTime]::UtcNow.ToString('o'))"
foreach ($path in $resolved) {
    Get-ChildItem -LiteralPath $path -File -Recurse | Where-Object {
        $_.Extension -in @('.exe', '.dll', '.zip')
    } | ForEach-Object {
        Write-ScanLog "SHA256 $((Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash) $($_.FullName)"
    }
}

$platform = Join-Path $env:ProgramData 'Microsoft\Windows Defender\Platform\*\MpCmdRun.exe'
$command = Get-ChildItem -Path $platform -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending | Select-Object -First 1 -ExpandProperty FullName
if (-not $command) { $command = Join-Path $env:ProgramFiles 'Windows Defender\MpCmdRun.exe' }
if (-not (Test-Path -LiteralPath $command)) { throw 'Microsoft Defender scanner is unavailable.' }
if ((Get-Service WinDefend).Status -ne 'Running') { Start-Service WinDefend }

Write-ScanLog 'Updating security intelligence from Microsoft...'
$updateOutput = & $command -SignatureUpdate -MMPC 2>&1
$updateCode = $LASTEXITCODE
$updateOutput | ForEach-Object { Write-ScanLog "$_" }
if ($updateCode -ne 0) { throw "Defender signature update failed: $updateCode. No clean verdict is available." }
$status = Get-MpComputerStatus
Write-ScanLog "Engine: $($status.AMEngineVersion); signatures: $($status.AntivirusSignatureVersion); updated: $($status.AntivirusSignatureLastUpdated.ToUniversalTime().ToString('o'))"
if (-not $status.AntivirusSignatureVersion -or
    $status.AntivirusSignatureLastUpdated -lt (Get-Date).AddDays(-2)) {
    throw 'Defender signatures are missing or stale. No clean verdict is available.'
}

# Hosted runners exclude build drives by default. -DisableRemediation ignores
# exclusions, scans archives, and leaves evidence unchanged (Microsoft's CLI).
# Verify that these custom scans actually detect the standard harmless EICAR
# test file before interpreting an application's scan as a no-detection result.
$controlDir = Join-Path ([IO.Path]::GetTempPath()) ('assdraw-av-control-' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $controlDir | Out-Null
try {
    $control = Join-Path $controlDir 'eicar.com.txt'
    Invoke-WebRequest -Uri 'https://secure.eicar.org/eicar.com.txt' -OutFile $control
    $expected = '275A021BBFB6489E54D471899F7DB9D1663FC695EC2FE2A2C4538AABF651FD0F'
    if ((Get-FileHash -LiteralPath $control -Algorithm SHA256).Hash -ne $expected) {
        throw 'The antivirus test fixture failed its SHA256 check.'
    }
    $controlOutput = & $command -Scan -ScanType 3 -File $control -DisableRemediation 2>&1
    $controlCode = $LASTEXITCODE
    $controlOutput | ForEach-Object { Write-ScanLog "$_" }
    if ($controlCode -ne 2 -or ($controlOutput -join "`n") -notmatch 'EICAR') {
        throw "The scanner did not detect the EICAR control (exit $controlCode). No clean verdict is available."
    }
    Write-ScanLog 'Positive control passed: the harmless EICAR test file was detected.'
} finally {
    Remove-Item -LiteralPath $controlDir -Recurse -Force
}

foreach ($path in $resolved) {
    Write-ScanLog "Scanning: $path"
    $scanOutput = & $command -Scan -ScanType 3 -File $path -DisableRemediation 2>&1
    $scanCode = $LASTEXITCODE
    $scanOutput | ForEach-Object { Write-ScanLog "$_" }
    if ($scanCode -ne 0) {
        throw "Defender reported a detection or scan error (exit $scanCode) for $path. See the scan report."
    }
    Write-ScanLog "Completed with no Defender detections: $path"
}
Write-ScanLog 'This result does not certify safety or determine Chrome/Avast verdicts.'
