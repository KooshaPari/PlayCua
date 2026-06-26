"""
laycua sandbox layer — Windows Sandbox / Hyper-V / Docker isolation for DINO test automation.

Provides:
  - SandboxConfig / MappedFolder dataclasses (config.py)
  - Sandboxfile YAML parser (sandboxfile.py)
  - Sandbox async context manager (sandbox.py)
  - GameSandbox — DINOForge-specific game launch + MCP bridge (game_sandbox.py)
  - SandboxPool — concurrent multi-instance orchestrator (pool.py)
  - SteamlessHelper — Steam DRM strip + base image management (steamless_helper.py)
  - PowerShell templates for BepInEx + SteamCMD setup (templates/)

Quickstart (WSB):
    from laycua.sandbox import GameSandbox

    async with GameSandbox.from_sandboxfile("DinoSandboxfile.yaml") as session:
        ok = await session.game_launch("main_menu")
        status = await session.game_status()
        print(status)

Quickstart (Hyper-V pool):
    from laycua.sandbox import SandboxPool

    async with SandboxPool(sandboxfile="DinoSandboxfile.yaml", max_concurrent=4) as pool:
        sessions = await pool.acquire(2)
        results = await asyncio.gather(*[s.game_launch() for s in sessions])
        await pool.release(sessions)

Steamless base image:
    from laycua.sandbox import SteamlessHelper

    helper = SteamlessHelper(
        source_game_dir="G:\\SteamLibrary\\steamapps\\common\\Diplomacy is Not an Option",
        base_image_dir="C:\\DINOForge\\base_images\\dino-stripped",
    )
    helper.download_steamless()
    helper.strip()
    helper.clone_to("C:\\HyperV\\VMs\\run-001")   # fast dir copy, no re-strip
"""

from .config import MappedFolder, SandboxConfig
from .sandbox import Computer, HyperVSandbox, Sandbox
from .sandboxfile import Sandboxfile, HealthCheck, Sandboxfile as SF
from .game_sandbox import GameSandbox, GameSandboxConfig, GameSandboxSession
from .pool import PooledSandbox, SandboxPool
from .steamless_helper import SteamlessHelper, SteamlessResult

__all__ = [
    # Config
    "MappedFolder",
    "SandboxConfig",
    # Base sandbox
    "Computer",
    "Sandbox",
    "HyperVSandbox",
    "Sandboxfile",
    "HealthCheck",
    # Game integration
    "GameSandbox",
    "GameSandboxConfig",
    "GameSandboxSession",
    # Pool
    "SandboxPool",
    "PooledSandbox",
    # Steamless
    "SteamlessHelper",
    "SteamlessResult",
]

