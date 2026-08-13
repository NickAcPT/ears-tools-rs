rustup run nightly wasm-pack build .\tools\bbmodel-generator\ --target web --out-name bbmodel_generator

if ($env:EARS_TOOLS_PATH) {
    $outputDir = Join-Path $env:EARS_TOOLS_PATH "src\tools\bbmodel-generator"
    $sourceDir = Join-Path (Get-Location) "tools\bbmodel-generator\pkg"
    
    # copy all files except package.json and .gitignore
    Get-ChildItem -Path $sourceDir -File | Where-Object { $_.Name -notin @("package.json", ".gitignore") } | ForEach-Object {
        Copy-Item -Path $_.FullName -Destination $outputDir -Force
    }
}