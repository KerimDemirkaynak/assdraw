param(
    [Parameter(Mandatory)]
    [string]$UnsignedExecutable
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$verify = Join-Path $PSScriptRoot '..\build\verify-windows-signature.ps1'

function Assert-Rejected([string]$File) {
    $rejected = $false
    try {
        & $verify -Path $File
    } catch {
        $rejected = $true
        Write-Host "Correctly rejected: $File ($($_.Exception.Message))"
    }
    if (-not $rejected) {
        throw "Signature verification incorrectly accepted $File."
    }
}

# PowerShell on the hosted Windows runner has a real Microsoft signature.
$trusted = Join-Path $PSHOME 'pwsh.exe'
& $verify -Path $trusted
Assert-Rejected $UnsignedExecutable

$scratch = Join-Path ([IO.Path]::GetTempPath()) ('assdraw-signature-test-' + [guid]::NewGuid())
New-Item -Path $scratch -ItemType Directory | Out-Null
try {
    Assert-Rejected (Join-Path $scratch 'missing.exe')
    $tampered = Join-Path $scratch 'tampered.exe'
    $bytes = [IO.File]::ReadAllBytes($trusted)
    # A reserved DOS-header byte is covered by Authenticode's file digest.
    $bytes[0x20] = $bytes[0x20] -bxor 1
    [IO.File]::WriteAllBytes($tampered, $bytes)
    Assert-Rejected $tampered
} finally {
    Remove-Item -LiteralPath $scratch -Recurse -Force
}
Write-Host 'Windows signature verification tests passed.'
