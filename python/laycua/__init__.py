from .computer import Computer
from .agent import ComputerAgent

# Sandbox layer (lazy import to avoid heavy deps)
try:
    from .sandbox import (
        GameSandbox,
        GameSandboxConfig,
        GameSandboxSession,
        HyperVSandbox,
        MappedFolder,
        Sandbox,
        SandboxConfig,
        SandboxPool,
        Sandboxfile,
        SteamlessHelper,
        SteamlessResult,
    )
except ImportError:
    # Sandbox module may not be available on all platforms
    GameSandbox = None  # type: ignore[assignment, misc]
    GameSandboxConfig = None  # type: ignore[assignment, misc]
    GameSandboxSession = None  # type: ignore[assignment, misc]
    HyperVSandbox = None  # type: ignore[assignment, misc]
    MappedFolder = None  # type: ignore[assignment, misc]
    Sandbox = None  # type: ignore[assignment, misc]
    SandboxConfig = None  # type: ignore[assignment, misc]
    SandboxPool = None  # type: ignore[assignment, misc]
    Sandboxfile = None  # type: ignore[assignment, misc]
    SteamlessHelper = None  # type: ignore[assignment, misc]
    SteamlessResult = None  # type: ignore[assignment, misc]

__all__ = [
    "Computer",
    "ComputerAgent",
    # Sandbox
    "GameSandbox",
    "GameSandboxConfig",
    "GameSandboxSession",
    "HyperVSandbox",
    "MappedFolder",
    "Sandbox",
    "SandboxConfig",
    "SandboxPool",
    "Sandboxfile",
    "SteamlessHelper",
    "SteamlessResult",
]

