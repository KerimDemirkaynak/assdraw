param(
    [string]$VcpkgRoot = '',
    [string]$VcpkgInstalledRoot = '',
    [string]$BuildDir = 'build-dir',
    [ValidateSet('Debug', 'Release', 'RelWithDebInfo')]
    [string]$Configuration = 'Release',
    [ValidateSet('x64-windows-release', 'x64-windows', 'x86-windows')]
    [string]$Triplet = 'x64-windows-release',
    [switch]$Fresh
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if (Get-Variable PSNativeCommandUseErrorActionPreference -ErrorAction SilentlyContinue) {
    $PSNativeCommandUseErrorActionPreference = $false
}

function Write-Step([string]$Message) {
    Write-Host ('==> ' + $Message) -ForegroundColor Cyan
}

function Get-VsGenerator {
    $vswhere = 'C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe'
    if (-not (Test-Path $vswhere)) { return 'Visual Studio 17 2022' }

    $installPath = & $vswhere -latest -products * -requires Microsoft.Component.MSBuild -property installationPath
    if (-not $installPath) { return 'Visual Studio 17 2022' }

    if ($installPath -match '\\18\\') { return 'Visual Studio 18 2026' }
    if ($installPath -match '\\17\\') { return 'Visual Studio 17 2022' }
    if ($installPath -match '\\16\\') { return 'Visual Studio 16 2019' }
    return 'Visual Studio 17 2022'
}

function Get-CMake {
    $cmake = Get-Command cmake.exe -ErrorAction SilentlyContinue
    if ($cmake) { return $cmake.Source }

    $vsCMake = @(
        'C:\Program Files\Microsoft Visual Studio\18\Community\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin\cmake.exe',
        'C:\Program Files\Microsoft Visual Studio\17\Community\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin\cmake.exe',
        'C:\Program Files\Microsoft Visual Studio\16\Community\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin\cmake.exe'
    ) | Where-Object { Test-Path $_ } | Select-Object -First 1

    if (-not $vsCMake) { throw 'cmake.exe not found.' }
    return $vsCMake
}

function Resolve-RepoPath([string]$PathValue) {
    if ([string]::IsNullOrWhiteSpace($PathValue)) {
        return ''
    }

    if ([System.IO.Path]::IsPathRooted($PathValue)) {
        return [System.IO.Path]::GetFullPath($PathValue)
    }

    return [System.IO.Path]::GetFullPath((Join-Path $repoRoot $PathValue))
}

function Invoke-Native([string]$FilePath, [string[]]$Arguments) {
    Write-Step (($FilePath, ($Arguments -join ' ')) -join ' ')
    & $FilePath @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "$FilePath failed with exit code $LASTEXITCODE."
    }
}

function Get-PlatformName([string]$ResolvedTriplet) {
    if ($ResolvedTriplet.StartsWith('x86-')) { return 'Win32' }
    return 'x64'
}

$repoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path | Split-Path -Parent
Set-Location $repoRoot

# The filename and old vcpkg parameters remain compatible with older commands.
# CMake now builds the same checksum-pinned wxWidgets 3.3.4 on both platforms.

$cmake = Get-CMake
$generator = Get-VsGenerator
$platform = Get-PlatformName $Triplet

if ($Fresh -and (Test-Path $BuildDir)) {
    Write-Step "Removing $BuildDir"
    Remove-Item $BuildDir -Recurse -Force
}

$cmakeArgs = [System.Collections.Generic.List[string]]::new()
foreach ($arg in @(
    '-S', '.',
    '-B', $BuildDir,
    '-G', $generator,
    '-A', $platform,
    '-DASSDRAW_BUNDLED_WX=ON'
)) {
    $cmakeArgs.Add($arg)
}

Invoke-Native $cmake $cmakeArgs

Invoke-Native $cmake @(
    '--build', $BuildDir,
    '--config', $Configuration,
    '--target', 'assdraw',
    '--parallel'
)

$exe = Join-Path $BuildDir "$Configuration\assdraw3.exe"
Write-Step "Done: $exe"
