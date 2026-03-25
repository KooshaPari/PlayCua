<#
.SYNOPSIS
    Install BepInEx into a DINO game directory inside a sandbox environment.

.DESCRIPTION
    This script is designed to run as the LogonCommand inside Windows Sandbox
    (or any fresh Windows environment). It:
      1. Copies BepInEx from the DINOForge repo (via mapped folder) into the game dir
      2. Copies DINOForge plugin DLLs into BepInEx/plugins/
      3. Copies mod packs into BepInEx/dinoforge_packs/
      4. Optionally installs the .NET 8.0 runtime if needed for tooling
      5. Writes a ready flag to the shared folder when complete

.PARAMETER GameDir
    Path to the DINO game directory (must contain Diplomacy is Not an Option.exe).

.PARAMETER DINOForgeRepo
    Path to the DINOForge repo (mapped read-only from host via .wsb MappedFolder).

.PARAMETER PluginBin
    Path to compiled DINOForge.Runtime.dll (mapped from host build output).

.PARAMETER SharedDir
    Path to the shared folder visible on host (for ready/error flag signaling).

.EXAMPLE
    powershell.exe -ExecutionPolicy Bypass -File setup_bepinex.ps1 `
        -GameDir "C:\DINO" `
        -DINOForgeRepo "C:\DINOForge" `
        -PluginBin "C:\DINOForge\bin" `
        -SharedDir "C:\SandboxShared"
#>
param(
    [string]$GameDir    = "C:\DINO",
    [string]$DINOForgeRepo = "C:\DINOForge",
    [string]$PluginBin  = "C:\DINOForge\bin",
    [string]$SharedDir  = "C:\SandboxShared",
    [string]$NativeBin  = "C:\SandboxInit\bare-cua-native.exe",
    [int]   $NativePort = 8765
)

$ErrorActionPreference = "Stop"
$ReadyFlag  = Join-Path $SharedDir "ready.flag"
$ErrorFlag  = Join-Path $SharedDir "ready.flag.error"

function Write-Step { param($msg) Write-Host "[SETUP] $msg" }
function Write-Ready { Set-Content -Path $ReadyFlag -Value (Get-Date -Format o) }
function Write-Error-Flag { param($msg) Set-Content -Path $ErrorFlag -Value $msg; exit 1 }

try {
    # --- Validate game directory ---
    $GameExe = Join-Path $GameDir "Diplomacy is Not an Option.exe"
    if (-not (Test-Path $GameExe)) {
        throw "Game executable not found at: $GameExe. Ensure DINO is mapped into $GameDir"
    }
    Write-Step "Game found: $GameExe"

    # --- BepInEx installation ---
    $BepInExSrc  = Join-Path $DINOForgeRepo "BepInEx"
    $BepInExDest = Join-Path $GameDir "BepInEx"

    if (Test-Path $BepInExSrc) {
        Write-Step "Copying BepInEx from $BepInExSrc to $BepInExDest"
        Copy-Item -Path $BepInExSrc -Destination $BepInExDest -Recurse -Force
    } else {
        throw "BepInEx source not found at $BepInExSrc"
    }

    # --- Plugin DLL ---
    $PluginDll = Join-Path $PluginBin "DINOForge.Runtime.dll"
    if (Test-Path $PluginDll) {
        $PluginDest = Join-Path $BepInExDest "plugins"
        New-Item -ItemType Directory -Force -Path $PluginDest | Out-Null
        Write-Step "Copying DINOForge.Runtime.dll to $PluginDest"
        Copy-Item -Path $PluginDll -Destination $PluginDest -Force

        # Copy any other DLLs from bin dir (dependencies)
        Get-ChildItem -Path $PluginBin -Filter "*.dll" | ForEach-Object {
            if ($_.Name -ne "DINOForge.Runtime.dll") {
                Copy-Item -Path $_.FullName -Destination $PluginDest -Force
            }
        }
    } else {
        Write-Warning "DINOForge.Runtime.dll not found at $PluginDll. Skipping plugin copy."
    }

    # --- Pack files ---
    $PacksSrc  = Join-Path $DINOForgeRepo "packs"
    $PacksDest = Join-Path $BepInExDest "dinoforge_packs"
    if (Test-Path $PacksSrc) {
        New-Item -ItemType Directory -Force -Path $PacksDest | Out-Null
        Write-Step "Copying packs from $PacksSrc to $PacksDest"
        Get-ChildItem -Path $PacksSrc -Directory | ForEach-Object {
            Copy-Item -Path $_.FullName -Destination $PacksDest -Recurse -Force
        }
    }

    # --- Write BepInEx doorstop config ---
    $DoorstopCfg = Join-Path $GameDir "doorstop_config.ini"
    if (-not (Test-Path $DoorstopCfg)) {
        Write-Step "Writing doorstop_config.ini"
        @"
[UnityDoorstop]
enabled=true
targetAssembly=BepInEx\core\BepInEx.dll
redirectOutputLog=false
"@ | Set-Content -Path $DoorstopCfg -Encoding UTF8
    }

    # --- Start bare-cua-native TCP listener (for Python host to connect) ---
    if (Test-Path $NativeBin) {
        Write-Step "Starting bare-cua-native on port $NativePort"
        Start-Process -FilePath $NativeBin -ArgumentList "--port", $NativePort -WindowStyle Hidden

        # Discover our IP on the Hyper-V Default Switch (NAT interface)
        $SandboxIP = (
            Get-NetIPAddress -AddressFamily IPv4 |
            Where-Object { $_.IPAddress -notmatch "^127\." -and $_.IPAddress -notmatch "^169\." } |
            Sort-Object -Property PrefixLength -Descending |
            Select-Object -First 1
        ).IPAddress

        if ($SandboxIP) {
            Write-Step "Sandbox IP: $SandboxIP"
            Set-Content -Path (Join-Path $SharedDir "sandbox_ip.txt") -Value $SandboxIP
        }
    } else {
        Write-Warning "bare-cua-native not found at $NativeBin. TCP IPC unavailable."
    }

    Write-Step "BepInEx setup complete."
    Write-Ready

} catch {
    Write-Error-Flag $_.Exception.Message
}
