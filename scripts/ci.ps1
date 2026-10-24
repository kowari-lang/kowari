#!/usr/bin/env pwsh
# Mirrors what CI runs (Planning.md §6.3).
$ErrorActionPreference = 'Stop'

function Invoke-Cargo {
    param([string[]] $CargoArgs)
    & cargo @CargoArgs
    if ($LASTEXITCODE -ne 0) {
        Write-Error "cargo $($CargoArgs -join ' ') failed (exit $LASTEXITCODE)"
        exit $LASTEXITCODE
    }
}

Invoke-Cargo @('fmt', '--all', '--check')
Invoke-Cargo @('clippy', '--workspace', '--all-targets', '--', '-D', 'warnings')
Invoke-Cargo @('test', '--workspace')
Invoke-Cargo @('build', '--workspace', '--release')