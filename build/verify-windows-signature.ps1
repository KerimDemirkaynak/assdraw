param(
    [Parameter(Mandatory)]
    [ValidateNotNullOrEmpty()]
    [string]$Path
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ($env:OS -ne 'Windows_NT') {
    throw 'Authenticode verification requires Windows.'
}
if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
    throw "Signed file is missing: $Path"
}

$file = (Resolve-Path -LiteralPath $Path).ProviderPath
$signature = Get-AuthenticodeSignature -FilePath $file
if ($signature.Status -ne 'Valid' -or $signature.SignatureType -ne 'Authenticode') {
    throw "A trusted embedded Authenticode signature is required for $file. Status: $($signature.Status); $($signature.StatusMessage)"
}
if ($null -eq $signature.SignerCertificate -or $null -eq $signature.TimeStamperCertificate) {
    throw "A signing certificate and timestamp are required for $file."
}

Write-Host "Verified timestamped Authenticode signature: $file"
Write-Host "Publisher: $($signature.SignerCertificate.Subject)"
Write-Host "Certificate thumbprint: $($signature.SignerCertificate.Thumbprint)"
Write-Host "File SHA256: $((Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash)"
