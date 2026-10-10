param([ValidateSet('dev','build','debug-build','installer','check','test','fmt','clippy')][string]$Action = 'build')
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path $PSScriptRoot -Parent
$cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
if (Test-Path $cargoBin) { $env:PATH = "$cargoBin;$env:PATH" }
# Use an optional project-local compiler. A normal Visual Studio installation needs no override.
$portableRoot = Join-Path $projectRoot 'tools\msvc'
if (Test-Path $portableRoot) {
    $msvcVersion = (Get-ChildItem "$portableRoot\VC\Tools\MSVC" -Directory | Select-Object -First 1).Name
    $sdkVersion = (Get-ChildItem "$portableRoot\Windows Kits\10\Lib" -Directory | Select-Object -First 1).Name
    $compiler = "$portableRoot\VC\Tools\MSVC\$msvcVersion"
    $sdk = "$portableRoot\Windows Kits\10"
    $env:PATH = "$compiler\bin\Hostx64\x64;$sdk\bin\$sdkVersion\x64;$env:PATH"
    $env:INCLUDE = "$compiler\include;$sdk\Include\$sdkVersion\ucrt;$sdk\Include\$sdkVersion\shared;$sdk\Include\$sdkVersion\um"
    $env:LIB = "$compiler\lib\x64;$sdk\Lib\$sdkVersion\ucrt\x64;$sdk\Lib\$sdkVersion\um\x64"
    $env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER = "$compiler\bin\Hostx64\x64\link.exe"
    $env:CC = "$compiler\bin\Hostx64\x64\cl.exe"
    $env:RC = "$sdk\bin\$sdkVersion\x64\rc.exe"
    $env:VCToolsInstallDir = "$compiler\"
    $env:WindowsSdkDir = "$sdk\"
    $env:WindowsSDKVersion = "$sdkVersion\"
}
Set-Location $projectRoot
switch ($Action) {
    'dev' { & npm.cmd run desktop:dev }
    'build' { & npm.cmd run desktop:build }
    'debug-build' { & npm.cmd run tauri -- build --debug --no-bundle }
    'installer' { & npm.cmd run desktop:installer }
    'check' { & cargo check --manifest-path src-tauri/Cargo.toml }
    'test' { & cargo test --manifest-path src-tauri/Cargo.toml }
    'fmt' { & cargo fmt --manifest-path src-tauri/Cargo.toml }
    'clippy' { & cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings }
}
exit $LASTEXITCODE
