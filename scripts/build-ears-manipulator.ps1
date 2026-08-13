rustup run nightly wasm-pack build .\tools\ears-manipulator\ --target web --out-name ears_manipulator

if ($env:EARS_TOOLS_PATH) {
    $outputDir = Join-Path $env:EARS_TOOLS_PATH "src\tools\ears-manipulator"
    $sourceDir = Join-Path (Get-Location) "tools\ears-manipulator\pkg"
    
    # copy all files except package.json and .gitignore
    Get-ChildItem -Path $sourceDir -File | Where-Object { $_.Name -notin @("package.json", ".gitignore") } | ForEach-Object {
        Copy-Item -Path $_.FullName -Destination $outputDir -Force
    }
}