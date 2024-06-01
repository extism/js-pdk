#!/usr/bin/env pwsh

$7z= "7z"
if (-not (Get-Command $7z -ErrorAction SilentlyContinue)){
  $7z= "$env:Programfiles\7-Zip\7z.exe"
}
$tempPath= "$env:TEMP"


if ((Split-Path -Leaf (Get-Location)) -ne "js-pdk") {
    Write-Error "Run this inside the root of the js-pdk repo"
    exit 1
}

if ($env:QUICKJS_WASM_SYS_WASI_SDK_PATH) {
    if (-Not (Test-Path -Path $env:QUICKJS_WASM_SYS_WASI_SDK_PATH)) {
        Write-Error "Download the wasi-sdk to $env:QUICKJS_WASM_SYS_WASI_SDK_PATH"
        exit 1
    }
    exit 0
}

$PATH_TO_SDK = "wasi-sdk"
if (-Not (Test-Path -Path $PATH_TO_SDK)) {
    $VERSION_MAJOR = "12"
    $VERSION_MINOR = "0"
    $url = "https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-$VERSION_MAJOR/wasi-sdk-$VERSION_MAJOR.$VERSION_MINOR-mingw.tar.gz"
    Invoke-WebRequest -Uri $url -OutFile "$tempPath\wasi-sdk-$VERSION_MAJOR.$VERSION_MINOR-mingw.tar.gz" 

    New-Item -ItemType Directory -Path $PATH_TO_SDK | Out-Null

    & $7z x "$tempPath\wasi-sdk-$VERSION_MAJOR.$VERSION_MINOR-mingw.tar.gz" -o"$tempPath" | Out-Null
    & $7z x -ttar "$tempPath\wasi-sdk-$VERSION_MAJOR.$VERSION_MINOR-mingw.tar" -o"$tempPath" | Out-Null

    Get-ChildItem -Path "$tempPath\wasi-sdk-$VERSION_MAJOR.$VERSION_MINOR" | ForEach-Object {
        Move-Item -Path $_.FullName -Destination $PATH_TO_SDK -Force | Out-Null
    }

    Remove-Item -Path "$tempPath\wasi-sdk-$VERSION_MAJOR.$VERSION_MINOR" -Recurse -Force | Out-Null
    Remove-Item -Path "$tempPath\wasi-sdk-$VERSION_MAJOR.$VERSION_MINOR-mingw.tar" -Recurse -Force | Out-Null
    Remove-Item -Path "$tempPath\wasi-sdk-$VERSION_MAJOR.$VERSION_MINOR-mingw.tar.gz" -Recurse -Force | Out-Null

}