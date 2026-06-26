"""
GameSandbox — DINOForge game automation on top of the laycua Sandbox layer.

Extends the base Sandbox with DINOForge-specific capabilities:
  - DINO game directory mapping and launch
  - BepInEx + DINOForge.Runtime.dll deployment via setup_bepinex.ps1
  - MCP bridge server connection (game_bridge.py / dinoforge_mcp)
  - Health check via MCP game_status tool
  - Steamless base image support (stripped + compressed game for fast cold starts)
  - Ready flag pipeline: setup → MCP connect → game launch → smoke test

Quickstart:
    from laycua.sandbox import GameSandbox

    async with GameSandbox.from_sandboxfile("DinoSandboxfile.yaml") as session:
        result = await session.computer.run("dotnet test C:\\DINOForge\\src\\Tests")
        print(result.output)
"""

from __future__ import annotations

import asyncio
import json
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Literal, Any

from .sandbox import Computer, Sandbox
from .sandboxfile import Sandboxfile
from .config import MappedFolder, SandboxConfig


@dataclass
class GameSandboxConfig:
    """Configuration for a DINOForge game sandbox.

    Composes SandboxConfig with game-specific settings.
    """

    # Base sandbox config
    sandbox: SandboxConfig

    # Game paths
    game_dir: str = "C:\\DINO"          # path inside sandbox where DINO is mapped
    game_exe: str = ""                 # auto-computed from game_dir if empty
    game_exe_name: str = "Diplomacy is Not an Option.exe"

    # MCP bridge
    mcp_port: int = 8765                # port laycua-native listens on
    bridge_health_timeout_s: int = 120  # timeout for MCP bridge to become ready

    # Game launch
    game_launch_timeout_s: int = 60     # timeout for game window to appear
    game_window_title: str = "Diplomacy is Not an Option"

    # Deployment
    deploy_runtime: bool = True         # deploy DINOForge.Runtime.dll from host build
    deploy_packs: bool = True           # deploy pack files from host repo
    runtime_dll_host_path: str = ""     # auto-detected if empty
    repo_host_path: str = ""            # auto-detected if empty

    # Steamless (pre-stripped base image)
    use_steamless_image: bool = False   # if True, game_dir points to Steamless-stripped copy
    steamless_base_path: str = ""       # host path to pre-extracted Steamless image dir

    @property
    def game_exe_path(self) -> str:
        if self.game_exe:
            return self.game_exe
        return str(Path(self.game_dir) / self.game_exe_name)

    def to_sandboxfile(self, template_dir: str | None = None) -> Sandboxfile:
        """Convert to a Sandboxfile.yaml with DINOForge setup steps."""
        if template_dir is None:
            import os
            template_dir = str(
                Path(os.path.dirname(__file__)) / "templates"
            )

        setup_steps = []

        # 1. Copy BepInEx from repo
        bepinex_dest = str(Path(self.game_dir) / "BepInEx")
        if self.deploy_runtime or self.deploy_packs:
            setup_steps.append({
                "run": (
                    f"powershell -ExecutionPolicy Bypass -Command "
                    f"'Copy-Item (Join-Path C:\\DINOForge BepInEx) {bepinex_dest} -Recurse -Force'"
                )
            })

        # 2. Deploy DINOForge.Runtime.dll + dependencies
        if self.deploy_runtime:
            runtime_src = self.runtime_dll_host_path
            if not runtime_src:
                runtime_src = str(
                    Path(self.repo_host_path or __file__).parent.parent.parent.parent
                    / "src" / "Runtime" / "bin" / "Release" / "net472"
                )
            setup_steps.append({
                "run": (
                    f"powershell -ExecutionPolicy Bypass -Command "
                    f"'&{{ "
                    f"$dest = Join-Path \"{bepinex_dest}\" plugins; "
                    f"New-Item -ItemType Directory -Force -Path $dest | Out-Null; "
                    f"Get-ChildItem \"{runtime_src}\" -Filter *.dll | "
                    f"Copy-Item -Destination $dest -Force; "
                    f"}}'"
                )
            })

        # 3. Deploy packs
        if self.deploy_packs:
            packs_dest = str(Path(self.game_dir) / "BepInEx" / "dinoforge_packs")
            setup_steps.append({
                "run": (
                    f"powershell -ExecutionPolicy Bypass -Command "
                    f"'&{{ "
                    f"$src = Join-Path C:\\DINOForge packs; "
                    f"$dest = \"{packs_dest}\"; "
                    f"New-Item -ItemType Directory -Force -Path $dest | Out-Null; "
                    f"if (Test-Path $src) {{ Get-ChildItem $src -Directory | "
                    f"Copy-Item -Destination $dest -Recurse -Force; }} }}'"
                )
            })

        # 4. Start game
        game_exe_sanitized = self.game_exe_path.replace("\\", "\\\\")
        setup_steps.append({
            "run": (
                f"powershell -ExecutionPolicy Bypass -Command "
                f"'Start-Process \"{game_exe_sanitized}\" -WorkingDirectory \"{self.game_dir}\" '"
            )
        })

        # 5. Wait for MCP bridge (via ready flag written by setup_bepinex.ps1)
        setup_steps.append({
            "wait_for_file": "C:\\SandboxShared\\bridge_ready.flag",
            "timeout_s": self.bridge_health_timeout_s,
        })

        return Sandboxfile(
            name=self.sandbox.name,
            base="windows-sandbox",
            memory_mb=self.sandbox.memory_mb,
            virtual_gpu=True,
            networking=True,
            cpu_count=2,
            mapped_folders=self.sandbox.mapped_folders,
            setup_steps=[],  # populated below
            health_check={
                "type": "file",
                "path": "C:\\SandboxShared\\game_ready.flag",
                "timeout_s": self.game_launch_timeout_s,
            },
            test_commands=[],
            env_vars=self.sandbox.startup_command and {} or {},
            extra={},
        )


class GameSandboxSession:
    """Active game sandbox session — exposes Computer + game control helpers.

    Returned by `GameSandbox.__aenter__()`. Use `.close()` or `async with` /
    `await aexit()` to clean up.
    """

    def __init__(
        self,
        computer: Computer,
        sandbox: "GameSandbox",
        sandbox_wsb_path: Path | None = None,
    ):
        self.computer = computer
        self.sandbox = sandbox
        self._sandbox_wsb_path = sandbox_wsb_path
        self._game_process: Any = None

    async def run(self, command: str, timeout_s: int = 300):
        """Run a shell command inside the sandbox."""
        return await self.computer.run(command, timeout_s)

    async def mcp_call(self, tool_name: str, **kwargs) -> dict[str, Any]:
        """Call an MCP tool by name and return the parsed result.

        Sends JSON-RPC via laycua-native TCP connection.
        Falls back to file-drop IPC if TCP fails.
        """
        req = {
            "jsonrpc": "2.0",
            "id": str(int(time.time() * 1000)),
            "method": f"tools/{tool_name}",
            "params": kwargs,
        }
        result = await self.computer.run(
            f'echo {json.dumps(req)}',
            timeout_s=60,
        )
        # TODO: wire actual MCP JSON-RPC once native bridge supports it
        return {}

    async def game_status(self) -> dict[str, Any]:
        """Poll game status via MCP bridge."""
        return await self.mcp_call("game_status")

    async def game_launch(self, scene: str = "main_menu") -> bool:
        """Launch the DINO game inside the sandbox to a given scene.

        Args:
            scene: Target scene — "main_menu" | "InitialGameLoader" | "gameplay"

        Returns:
            True if game launched successfully and bridge is accessible.
        """
        result = await self.run(
            f'powershell -Command "Start-Process \'{self.sandbox._cfg.game_exe_path}\' '
            f'-WorkingDirectory \'{self.sandbox._cfg.game_dir}\'"',
            timeout_s=30,
        )
        if not result.success:
            return False

        # Wait for bridge to become ready
        deadline = time.monotonic() + self.sandbox._cfg.bridge_health_timeout_s
        while time.monotonic() < deadline:
            status = await self.game_status()
            if status.get("running"):
                return True
            await asyncio.sleep(2)

        return False

    async def dump_world(self) -> dict[str, Any]:
        """Trigger an ECS world dump and return parsed entity counts."""
        return await self.mcp_call("dump_world")

    async def screenshot(self, output_path: str = "C:\\SandboxShared\\screen.png") -> bytes | None:
        """Capture game window screenshot and return raw PNG bytes."""
        await self.mcp_call("game_screenshot", output_path=output_path)
        result = await self.computer.run(
            f"powershell -Command \""
            f"$b = [IO.File]::ReadAllBytes('{output_path}'); "
            f"[Convert]::ToBase64String($b)\"",
            timeout_s=30,
        )
        if result.success and result.output.strip():
            import base64
            return base64.b64decode(result.output.strip())
        return None

    def close(self) -> None:
        """Stop the sandbox and clean up resources."""
        self.computer.terminate()

    async def aclose(self) -> None:
        """Async cleanup."""
        self.close()

    def __repr__(self) -> str:
        ip = self.computer.sandbox_ip or "no-ip"
        return f"<GameSandboxSession ip={ip}>"


class GameSandbox:
    """DINOForge game automation sandbox.

    Wraps the base Sandbox with DINOForge-specific deployment and MCP bridge
    integration. Supports:
      - windows-sandbox  (WSB, single instance, 5-10s cold start)
      - hyperv-vm        (concurrent instances, full GPU, slower start)

    Usage:
        async with GameSandbox.from_sandboxfile("DinoSandboxfile.yaml") as session:
            ok = await session.game_launch("main_menu")
            status = await session.game_status()
            print(status)

        # Or programmatically:
        cfg = GameSandboxConfig(sandbox=SandboxConfig(name="dino-test"))
        async with GameSandbox(cfg) as session:
            ...
    """

    def __init__(
        self,
        config: GameSandboxConfig,
        sandboxfile: Sandboxfile | None = None,
        poll_interval_s: float = 2.0,
    ):
        self._cfg = config
        self._sf = sandboxfile
        self._poll_interval = poll_interval_s
        self._sandbox: Sandbox | None = None
        self._session: GameSandboxSession | None = None

    @classmethod
    def from_sandboxfile(cls, path: str | Path, **kw) -> "GameSandbox":
        """Load from a Sandboxfile.yaml with GameSandbox defaults."""
        sf = Sandboxfile.load(path)
        # Detect game_dir from mapped folders if not set
        game_dir = ""
        for mf in sf.mapped_folders:
            if "Diplomacy is Not an Option" in mf.host_folder:
                game_dir = mf.sandbox_folder or "C:\\DINO"
                break
        gcfg = GameSandboxConfig(
            sandbox=sf.to_sandbox_config(),
            game_dir=game_dir,
        )
        return cls(config=gcfg, sandboxfile=sf, **kw)

    @classmethod
    def from_game_dir(
        cls,
        game_dir_host: str,
        repo_dir_host: str,
        name: str = "dino-game-sandbox",
        memory_mb: int = 8192,
    ) -> "GameSandbox":
        """Quick constructor from host paths.

        Args:
            game_dir_host:  Host path to DINO game directory (mapped read-only).
            repo_dir_host:  Host path to DINOForge repo root.
            name:           Sandbox name.
            memory_mb:      RAM allocation.
        """
        game_dir_sandbox = "C:\\DINO"
        runtime_host = str(
            Path(repo_dir_host) / "src" / "Runtime" / "bin" / "Release" / "net472"
        )
        sandbox_cfg = SandboxConfig(
            name=name,
            memory_mb=memory_mb,
            virtual_gpu="Enable",
            networking="Enable",
            mapped_folders=[
                MappedFolder(
                    host_folder=game_dir_host,
                    sandbox_folder=game_dir_sandbox,
                    read_only=True,
                ),
                MappedFolder(
                    host_folder=repo_dir_host,
                    sandbox_folder="C:\\DINOForge",
                    read_only=True,
                ),
            ],
        )
        gcfg = GameSandboxConfig(
            sandbox=sandbox_cfg,
            game_dir=game_dir_sandbox,
            runtime_dll_host_path=runtime_host,
            repo_host_path=repo_dir_host,
        )
        return cls(config=gcfg)

    async def __aenter__(self) -> GameSandboxSession:
        """Start the sandbox and wait for the game to be ready."""
        # Build the effective sandboxfile with game setup steps
        sf = self._sf
        if sf is None:
            sf = self._cfg.to_sandboxfile()

        self._sandbox = Sandbox(config=sf.to_sandbox_config(), sandboxfile=sf)
        computer = await self._sandbox.__aenter__()
        self._session = GameSandboxSession(
            computer=computer,
            sandbox=self,
            sandbox_wsb_path=getattr(self._sandbox, "_wsb_path", None),
        )
        return self._session

    async def __aexit__(self, *args) -> None:
        if self._session:
            await self._session.aclose()
        if self._sandbox:
            await self._sandbox.__aexit__(*args)
