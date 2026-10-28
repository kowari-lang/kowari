<#
.SYNOPSIS
    Regenerates AST golden files for kowari-test-harness.
.DESCRIPTION
    Sets UPDATE=1, runs the ignored ast_golden test, then clears the
    environment variable. The try/finally block guarantees the variable
    is removed even if cargo test returns a non-zero exit code.
#>

try {
    $env:UPDATE = "1"
    cargo test -p kowari-test-harness -- --ignored

    if ($LASTEXITCODE -ne 0) {
        Write-Error "cargo test failed with exit code $LASTEXITCODE"
        exit $LASTEXITCODE
    }
}
finally {
    Remove-Item Env:UPDATE -ErrorAction SilentlyContinue
}