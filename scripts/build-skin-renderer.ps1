$features = @("webgpu", "webgl", "software-rendering"); foreach ($feature in $features) { $command = "rustup run nightly wasm-pack build .\tools\skin-renderer\ --target web --out-name skin-renderer-$($feature.Split("-")[0])_wasm --features $($feature) "; Invoke-Expression $command; }

if ($env:EARS_TOOLS_PATH) {
    $outputDir = Join-Path $env:EARS_TOOLS_PATH "src\tools\skin-renderer"
    $sourceDir = Join-Path (Get-Location) "tools\skin-renderer\pkg"
    
    # copy all files except package.json and .gitignore
    Get-ChildItem -Path $sourceDir -File | Where-Object { $_.Name -notin @("package.json", ".gitignore") } | ForEach-Object {
        Copy-Item -Path $_.FullName -Destination $outputDir -Force
        Write-Host "Copied $($_.Name) to $outputDir"
    }
}