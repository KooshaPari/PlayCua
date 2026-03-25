<#
.SYNOPSIS
    Install "Diplomacy is Not an Option" via SteamCMD in a sandbox environment.

.DESCRIPTION
    Downloads SteamCMD, authenticates, and installs DINO (AppID 1273720).
    Designed for use inside Windows Sandbox or Hyper-V VMs for CI/CD pipelines.

    Steam App ID for DINO: 1273720

    CREDENTIAL HANDLING:
    ===================
    DINO is not a free-to-play game and requires a Steam account that owns it.
    SteamCMD cannot be used anonymously for this title.

    Three credential strategies in order of preference:

    1. Pre-cached credentials (RECOMMENDED for CI):
       - Log in interactively once on a dedicated Steam account
       - SteamCMD stores ssfn* files in $SteamCmdDir\config\
       - Map that config dir read-only into the sandbox
       - Use: -SteamUser "username" (no password) for subsequent runs
       - Credentials stay valid for days on the same machine/IP

    2. Environment variables (for pipelines):
       - $env:STEAM_USER, $env:STEAM_PASSWORD
       - First run requires Steam Guard code (email or TOTP)
       - For TOTP: use steamcmd-2fa (github.com/Weilbyte/steamcmd-2fa)
         to pre-generate the code from the shared secret

    3. Windows Credential Manager:
       - Store with: cmdkey /generic:SteamCMD /user:<user> /pass:<pass>
       - Retrieve with: (Get-StoredCredential -Target SteamCMD).GetNetworkCredential()
       - Requires CredentialManager module: Install-Module -Name CredentialManager

    2FA NOTES:
    ==========
    - Steam Guard via email: first login sends code, subsequent logins reuse cache
    - Steam Guard via mobile (TOTP): requires shared_secret from account setup
      - Tool: steamcmd-2fa can auto-generate TOTP codes non-interactively
      - Inside CI sandbox: export TOTP secret to env var, use steamcmd-2fa wrapper
    - If running on a fixed IP, Steam Guard codes are not re-requested frequently

.PARAMETER SteamUser
    Steam account username. REQUIRED.

.PARAMETER SteamPassword
    Steam account password. Optional if cached credentials exist.

.PARAMETER SteamGuardCode
    Steam Guard code (email or TOTP). Optional if cache is warm.

.PARAMETER InstallPath
    Where to install the game inside the sandbox.

.PARAMETER SteamCmdDir
    Where to download and run SteamCMD from.

.PARAMETER SharedDir
    Shared folder path for ready/error flag signaling back to Python host.

.PARAMETER PreCachedConfigDir
    Optional: path to a pre-populated SteamCMD config/ dir with ssfn* credentials.
    Map this read-only from host: steamcmd_cache -> C:\SteamCmdCache

.EXAMPLE
    # Using cached credentials (no password needed):
    powershell.exe -ExecutionPolicy Bypass -File install_dino.ps1 `
        -SteamUser "myaccount" `
        -InstallPath "C:\DINO" `
        -PreCachedConfigDir "C:\SteamCmdCache"

.EXAMPLE
    # First-time install with password (will prompt for Steam Guard if needed):
    powershell.exe -ExecutionPolicy Bypass -File install_dino.ps1 `
        -SteamUser $env:STEAM_USER `
        -SteamPassword $env:STEAM_PASS `
        -InstallPath "C:\DINO"
#>
param(
    [Parameter(Mandatory = $true)]
    [string]$SteamUser,

    [string]$SteamPassword       = $env:STEAM_PASSWORD,
    [string]$SteamGuardCode      = $env:STEAM_GUARD_CODE,
    [string]$InstallPath         = "C:\DINO",
    [string]$SteamCmdDir         = "C:\steamcmd",
    [string]$SharedDir           = "C:\SandboxShared",
    [string]$PreCachedConfigDir  = "C:\SteamCmdCache",
    [int]   $AppId               = 1273720,
    [int]   $TimeoutMinutes      = 60
)

$ErrorActionPreference = "Stop"
$ReadyFlag  = Join-Path $SharedDir "ready.flag"
$ErrorFlag  = Join-Path $SharedDir "ready.flag.error"
$SteamCmdExe = Join-Path $SteamCmdDir "steamcmd.exe"

function Write-Step { param($msg) Write-Host "[INSTALL] $msg" }
function Write-Ready { Set-Content -Path $ReadyFlag -Value (Get-Date -Format o) }
function Write-Error-Flag { param($msg) Set-Content -Path $ErrorFlag -Value $msg; exit 1 }

try {
    # --- Download SteamCMD if not present ---
    if (-not (Test-Path $SteamCmdExe)) {
        Write-Step "Downloading SteamCMD to $SteamCmdDir"
        New-Item -ItemType Directory -Force -Path $SteamCmdDir | Out-Null
        $ZipPath = Join-Path $env:TEMP "steamcmd.zip"
        Invoke-WebRequest -Uri "https://steamcdn-a.akamaihd.net/client/installer/steamcmd.zip" `
            -OutFile $ZipPath -UseBasicParsing
        Expand-Archive -Path $ZipPath -DestinationPath $SteamCmdDir -Force
        Remove-Item $ZipPath -Force
        Write-Step "SteamCMD downloaded to $SteamCmdExe"
    } else {
        Write-Step "SteamCMD already present at $SteamCmdExe"
    }

    # --- Restore pre-cached credentials if available ---
    if (Test-Path $PreCachedConfigDir) {
        Write-Step "Restoring cached SteamCMD credentials from $PreCachedConfigDir"
        $DestConfig = Join-Path $SteamCmdDir "config"
        New-Item -ItemType Directory -Force -Path $DestConfig | Out-Null
        Copy-Item -Path (Join-Path $PreCachedConfigDir "*") -Destination $DestConfig -Recurse -Force
    }

    # --- Build SteamCMD command script ---
    # We write a .txt script file and pass it to steamcmd via +runscript
    # This avoids shell injection from credential strings
    $ScriptContent = @"
@ShutdownOnFailedCommand 1
@NoPromptForPassword 1
"@

    # Login line: only include password if provided (cached credentials use username only)
    if ($SteamPassword) {
        if ($SteamGuardCode) {
            $ScriptContent += "login $SteamUser $SteamPassword $SteamGuardCode`n"
        } else {
            $ScriptContent += "login $SteamUser $SteamPassword`n"
        }
    } else {
        Write-Step "No password provided -- attempting login with cached credentials"
        $ScriptContent += "login $SteamUser`n"
    }

    $ScriptContent += "force_install_dir `"$InstallPath`"`n"
    $ScriptContent += "app_update $AppId validate`n"
    $ScriptContent += "quit`n"

    $ScriptFile = Join-Path $env:TEMP "install_dino.txt"
    Set-Content -Path $ScriptFile -Value $ScriptContent -Encoding UTF8
    Write-Step "SteamCMD script written to $ScriptFile"

    # --- Create install directory ---
    New-Item -ItemType Directory -Force -Path $InstallPath | Out-Null

    # --- Run SteamCMD ---
    Write-Step "Starting DINO installation (AppID $AppId) to $InstallPath"
    Write-Step "This may take 30-60 minutes depending on download speed (~6GB)"

    $proc = Start-Process -FilePath $SteamCmdExe `
        -ArgumentList "+runscript `"$ScriptFile`"" `
        -PassThru -NoNewWindow -Wait

    if ($proc.ExitCode -ne 0) {
        throw "SteamCMD exited with code $($proc.ExitCode). Check credentials and network."
    }

    # --- Verify installation ---
    $GameExe = Join-Path $InstallPath "Diplomacy is Not an Option.exe"
    if (-not (Test-Path $GameExe)) {
        throw "Installation verification failed: game exe not found at $GameExe"
    }
    $GameSize = (Get-Item $GameExe).Length
    Write-Step "Game exe found ($($GameSize / 1MB) MB): $GameExe"

    # Clean up script file
    Remove-Item $ScriptFile -Force -ErrorAction SilentlyContinue

    Write-Step "DINO installation complete."
    Write-Ready

} catch {
    Write-Error-Flag $_.Exception.Message
}
