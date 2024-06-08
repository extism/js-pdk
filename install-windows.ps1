#!/usr/bin/env pwsh

$TAG= "v1.0.0-rc11"
$BINARYEN_TAG= "version_116"
$extismPath="$env:Programfiles\Extism"
$binaryenPath="$env:Programfiles\Binaryen"
$7z= "7z"
if (-not (Get-Command $7z -ErrorAction SilentlyContinue)){
  $7z= "$env:Programfiles\7-Zip\7z.exe"
}
$tempPath= "$env:TEMP"

try {

    $ARCH = (Get-CimInstance Win32_Processor).Architecture
    switch ($ARCH) {
        9 { $ARCH = "x86_64" }
        12 { $ARCH = "aarch64" }
        default { Write-Host "unknown arch: $ARCH" -ForegroundColor Red; exit 1 }
    }
    Write-Host "ARCH is $ARCH."

    Write-Host "Downloading extism-js version $TAG."
    $TMPGZ = [System.IO.Path]::GetTempFileName()
    Invoke-WebRequest -Uri "https://github.com/extism/js-pdk/releases/download/$TAG/extism-js-$ARCH-windows-$TAG.gz" -OutFile "$TMPGZ"

    Write-Host "Installing extism-js."
    Remove-Item -Recurse -Path "$extismPath" -Force -ErrorAction SilentlyContinue | Out-Null
    New-Item -ItemType Directory -Force -Path $extismPath -ErrorAction Stop | Out-Null
    & $7z x "$TMPGZ" -o"$extismPath" >$null  2>&1

    if (-not (Get-Command "wasm-merge" -ErrorAction SilentlyContinue) -or -not (Get-Command "wasm-opt" -ErrorAction SilentlyContinue)) {
    
        Write-Output "Missing Binaryen tool(s)."
        Remove-Item -Recurse -Path "$binaryenPath" -Force -ErrorAction SilentlyContinue | Out-Null
        New-Item -ItemType Directory -Force -Path $binaryenPath -ErrorAction Stop | Out-Null
        
        Write-Output "Downloading Binaryen version $BINARYEN_TAG."
        Remove-Item -Recurse -Path "$tempPath\binaryen-*" -Force -ErrorAction SilentlyContinue | Out-Null
        Invoke-WebRequest -Uri "https://github.com/WebAssembly/binaryen/releases/download/$BINARYEN_TAG/binaryen-$BINARYEN_TAG-$ARCH-windows.tar.gz" -OutFile "$tempPath\binaryen-$BINARYEN_TAG-$ARCH-windows.tar.gz"

    
        Write-Output "Installing Binaryen."
        & $7z x "$tempPath\binaryen-$BINARYEN_TAG-$ARCH-windows.tar.gz" -o"$tempPath" >$null  2>&1
        & $7z x -ttar "$tempPath\binaryen-$BINARYEN_TAG-$ARCH-windows.tar" -o"$tempPath" >$null  2>&1
        Copy-Item "$tempPath\binaryen-$BINARYEN_TAG\bin\wasm-opt.exe" -Destination "$binaryenPath" -ErrorAction Stop | Out-Null
        Copy-Item "$tempPath\binaryen-$BINARYEN_TAG\bin\wasm-merge.exe" -Destination "$binaryenPath" -ErrorAction Stop | Out-Null
    }

    Write-Output "Install done !"
}catch {
  Write-Output "Install Failed: $_.Exception.Message"
  exit 1
}
