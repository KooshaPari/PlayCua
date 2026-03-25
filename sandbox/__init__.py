"""
bare-cua sandbox layer — Windows Sandbox / Hyper-V / Docker isolation for DINO test automation.

Provides:
  - SandboxConfig / MappedFolder dataclasses (config.py)
  - Sandboxfile YAML parser (sandboxfile.py)
  - Sandbox async context manager (sandbox.py)
  - PowerShell templates for BepInEx + SteamCMD setup (templates/)

Quickstart:
    from bare_cua.sandbox import Sandbox

    async with Sandbox.from_sandboxfile("Sandboxfile.yaml") as computer:
        result = await computer.run("dotnet test C:\\DINOForge\\src\\Tests")
        print(result.output)
"""

from .config import MappedFolder, SandboxConfig
from .sandbox import Sandbox
from .sandboxfile import Sandboxfile

__all__ = [
    "MappedFolder",
    "SandboxConfig",
    "Sandbox",
    "Sandboxfile",
]
