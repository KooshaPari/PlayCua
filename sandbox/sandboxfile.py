"""
Sandboxfile — Dockerfile-like declarative sandbox specification.

A Sandboxfile.yaml describes a complete, reproducible sandbox environment:
  - What type of isolation to use (windows-sandbox | hyperv-vm | docker-windows)
  - Resource limits (memory, CPU, GPU)
  - Folder mappings from host into the sandbox
  - Ordered setup steps executed after the sandbox boots
  - Health check to determine when the sandbox is ready
  - Test commands to run inside the sandbox

Example Sandboxfile.yaml:

    name: dino-test-env
    base: windows-sandbox
    memory_mb: 8192
    virtual_gpu: true
    networking: true

    map:
      - host: "G:\\SteamLibrary\\steamapps\\common\\Diplomacy is Not an Option"
        sandbox: "C:\\DINO"
        readonly: true
      - host: "C:\\Users\\koosh\\Dino\\src\\Runtime\\bin\\Release\\net472"
        sandbox: "C:\\DINO\\BepInEx\\plugins"
        readonly: false

    setup:
      - run: "powershell -c 'Copy-Item C:\\DINOForge\\BepInEx -Destination C:\\DINO\\BepInEx -Recurse'"
      - run: "C:\\DINO\\install_bepinex.ps1"
      - wait_for_file: "C:\\DINO\\BepInEx\\LogOutput.log"

    health_check:
      type: file
      path: "C:\\ready.flag"
      timeout_s: 120

    test:
      - run: "dotnet test C:\\DINOForge\\src\\Tests"
"""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Literal

import yaml

from .config import MappedFolder, SandboxConfig


BaseImage = Literal["windows-sandbox", "hyperv-vm", "docker-windows"]


# ---------------------------------------------------------------------------
# Setup step types
# ---------------------------------------------------------------------------


@dataclass
class RunStep:
    """Execute a shell command inside the sandbox."""

    command: str

    def __str__(self) -> str:
        return f"run: {self.command}"


@dataclass
class WaitForFileStep:
    """Block sandbox setup until a file appears (poll every 2s)."""

    path: str
    timeout_s: int = 120

    def __str__(self) -> str:
        return f"wait_for_file: {self.path} (timeout={self.timeout_s}s)"


@dataclass
class WaitForProcessStep:
    """Block sandbox setup until a named process is running."""

    process_name: str
    timeout_s: int = 120

    def __str__(self) -> str:
        return f"wait_for_process: {self.process_name} (timeout={self.timeout_s}s)"


@dataclass
class CopyStep:
    """Copy a file from a sandbox path to another sandbox path."""

    src: str
    dst: str

    def __str__(self) -> str:
        return f"copy: {self.src} -> {self.dst}"


SetupStep = RunStep | WaitForFileStep | WaitForProcessStep | CopyStep


# ---------------------------------------------------------------------------
# Health check
# ---------------------------------------------------------------------------


@dataclass
class HealthCheck:
    """Defines when a sandbox is considered "ready" for use.

    type:
      file      — wait until a file exists at `path`
      process   — wait until a process named `name` appears
      tcp       — wait until a TCP port on the sandbox IP responds
      pipe      — wait until a named pipe \\.\pipe\<name> is connectable from host
    """

    type: Literal["file", "process", "tcp", "pipe"]
    path: str | None = None          # for type=file
    name: str | None = None          # for type=process or type=pipe
    host: str | None = None          # for type=tcp (sandbox IP or hostname)
    port: int | None = None          # for type=tcp
    timeout_s: int = 120

    @staticmethod
    def from_dict(d: dict) -> "HealthCheck":
        return HealthCheck(
            type=d["type"],
            path=d.get("path"),
            name=d.get("name"),
            host=d.get("host"),
            port=d.get("port"),
            timeout_s=int(d.get("timeout_s", 120)),
        )

    def describe(self) -> str:
        if self.type == "file":
            return f"file exists: {self.path}"
        if self.type == "process":
            return f"process running: {self.name}"
        if self.type == "tcp":
            return f"TCP connect: {self.host}:{self.port}"
        if self.type == "pipe":
            return f"named pipe: \\\\.\\pipe\\{self.name}"
        return f"health_check(type={self.type})"


# ---------------------------------------------------------------------------
# Sandboxfile
# ---------------------------------------------------------------------------


@dataclass
class Sandboxfile:
    """Parsed representation of a Sandboxfile.yaml.

    The Sandboxfile is the single source of truth for a reproducible sandbox
    environment. It encodes enough information to:
      1. Generate a .wsb file (for windows-sandbox base)
      2. Generate Hyper-V PowerShell commands (for hyperv-vm base)
      3. Generate a docker run command (for docker-windows base)
      4. Render a startup PowerShell script that executes setup steps
      5. Drive the health-check poll loop
      6. Run the declared test suite
    """

    name: str
    base: BaseImage
    memory_mb: int
    virtual_gpu: bool
    networking: bool
    cpu_count: int
    mapped_folders: list[MappedFolder]
    setup_steps: list[SetupStep]
    health_check: HealthCheck
    test_commands: list[str]
    env_vars: dict[str, str]
    extra: dict[str, Any]

    # ------------------------------------------------------------------
    # Parsing
    # ------------------------------------------------------------------

    @staticmethod
    def load(path: str | Path) -> "Sandboxfile":
        """Load and validate a Sandboxfile.yaml from disk."""
        with open(path, "r", encoding="utf-8") as fh:
            data = yaml.safe_load(fh)
        return Sandboxfile._from_dict(data)

    @staticmethod
    def loads(text: str) -> "Sandboxfile":
        """Parse a Sandboxfile from a YAML string."""
        data = yaml.safe_load(text)
        return Sandboxfile._from_dict(data)

    @staticmethod
    def _from_dict(data: dict) -> "Sandboxfile":
        name = data.get("name", "sandbox")
        base: BaseImage = data.get("base", "windows-sandbox")  # type: ignore[assignment]
        memory_mb = int(data.get("memory_mb", 4096))
        virtual_gpu = bool(data.get("virtual_gpu", True))
        networking = bool(data.get("networking", True))
        cpu_count = int(data.get("cpu_count", 2))

        mapped: list[MappedFolder] = [
            MappedFolder.from_dict(m) for m in data.get("map", [])
        ]

        setup_steps: list[SetupStep] = []
        for step in data.get("setup", []):
            if isinstance(step, str):
                setup_steps.append(RunStep(step))
            elif "run" in step:
                setup_steps.append(RunStep(step["run"]))
            elif "wait_for_file" in step:
                timeout = step.get("timeout_s", 120)
                setup_steps.append(WaitForFileStep(step["wait_for_file"], timeout))
            elif "wait_for_process" in step:
                timeout = step.get("timeout_s", 120)
                setup_steps.append(WaitForProcessStep(step["wait_for_process"], timeout))
            elif "copy" in step:
                parts = step["copy"].split("->")
                if len(parts) == 2:
                    setup_steps.append(CopyStep(parts[0].strip(), parts[1].strip()))
                else:
                    raise ValueError(f"Invalid copy step syntax: {step!r}")
            else:
                raise ValueError(f"Unknown setup step: {step!r}")

        hc_data = data.get("health_check", {"type": "file", "path": "C:\\ready.flag"})
        health_check = HealthCheck.from_dict(hc_data)

        test_commands: list[str] = []
        for t in data.get("test", []):
            if isinstance(t, str):
                test_commands.append(t)
            elif "run" in t:
                test_commands.append(t["run"])

        env_vars: dict[str, str] = data.get("env", {})

        # Preserve any unknown keys for forward compatibility
        known = {
            "name", "base", "memory_mb", "virtual_gpu", "networking", "cpu_count",
            "map", "setup", "health_check", "test", "env",
        }
        extra = {k: v for k, v in data.items() if k not in known}

        return Sandboxfile(
            name=name,
            base=base,
            memory_mb=memory_mb,
            virtual_gpu=virtual_gpu,
            networking=networking,
            cpu_count=cpu_count,
            mapped_folders=mapped,
            setup_steps=setup_steps,
            health_check=health_check,
            test_commands=test_commands,
            env_vars=env_vars,
            extra=extra,
        )

    # ------------------------------------------------------------------
    # Rendering
    # ------------------------------------------------------------------

    def to_sandbox_config(self) -> SandboxConfig:
        """Convert to SandboxConfig (for windows-sandbox base only).

        Raises ValueError if base != 'windows-sandbox'.
        """
        if self.base != "windows-sandbox":
            raise ValueError(
                f"to_sandbox_config() is only valid for base=windows-sandbox, "
                f"got base={self.base!r}"
            )
        return SandboxConfig(
            name=self.name,
            memory_mb=self.memory_mb,
            virtual_gpu="Enable" if self.virtual_gpu else "Disable",
            networking="Enable" if self.networking else "Disable",
            mapped_folders=self.mapped_folders,
            # Startup script is rendered by Sandbox.run() and injected at launch time
        )

    def render_startup_script(self, ready_flag_path: str = "C:\\ready.flag") -> str:
        """Render a PowerShell script that executes all setup steps in order.

        The script:
          1. Sets a global error preference
          2. Runs each setup step
          3. Writes a ready-flag file when all steps complete
          4. Writes an error-flag file if any step fails

        The health-check in the Python host polls for the ready-flag.
        """
        lines = [
            "$ErrorActionPreference = 'Stop'",
            "$ReadyFlag = '{}'".format(ready_flag_path),
            "$ErrorFlag = '{}.error'".format(ready_flag_path),
            "",
            "try {",
        ]

        # Environment variables
        for key, value in self.env_vars.items():
            lines.append(f'    $env:{key} = "{value}"')

        for step in self.setup_steps:
            if isinstance(step, RunStep):
                # Escape double-quotes in command
                cmd = step.command.replace('"', '`"')
                lines.append(f"    Write-Host 'STEP: {step}'")
                lines.append(f"    Invoke-Expression \"{cmd}\"")
            elif isinstance(step, WaitForFileStep):
                lines.append(f"    Write-Host 'WAIT: {step}'")
                lines.append(f"    $deadline = (Get-Date).AddSeconds({step.timeout_s})")
                lines.append(f"    while (-not (Test-Path '{step.path}')) {{")
                lines.append(
                    "        if ((Get-Date) -gt $deadline) { throw 'Timeout waiting for "
                    f"{step.path}' }}"
                )
                lines.append("        Start-Sleep -Seconds 2")
                lines.append("    }")
            elif isinstance(step, WaitForProcessStep):
                lines.append(f"    Write-Host 'WAIT: {step}'")
                lines.append(f"    $deadline = (Get-Date).AddSeconds({step.timeout_s})")
                lines.append(
                    f"    while (-not (Get-Process '{step.process_name}' -ErrorAction SilentlyContinue)) {{"
                )
                lines.append(
                    "        if ((Get-Date) -gt $deadline) { throw 'Timeout waiting for process "
                    f"{step.process_name}' }}"
                )
                lines.append("        Start-Sleep -Seconds 2")
                lines.append("    }")
            elif isinstance(step, CopyStep):
                lines.append(f"    Write-Host 'COPY: {step}'")
                lines.append(
                    f"    Copy-Item -Path '{step.src}' -Destination '{step.dst}' -Recurse -Force"
                )

        lines.append("")
        lines.append(f"    Set-Content -Path $ReadyFlag -Value (Get-Date -Format o)")
        lines.append("    Write-Host 'SANDBOX_READY'")
        lines.append("} catch {")
        lines.append(
            "    Set-Content -Path $ErrorFlag -Value $_.Exception.Message"
        )
        lines.append("    Write-Host \"SANDBOX_ERROR: $_\"")
        lines.append("    exit 1")
        lines.append("}")

        return "\r\n".join(lines)

    def to_hyperv_script(
        self,
        vm_name: str | None = None,
        base_vhdx: str = "C:\\HyperV\\BaseImages\\Windows11.vhdx",
        vm_dir: str = "C:\\HyperV\\VMs",
        switch_name: str = "Default Switch",
    ) -> str:
        """Render PowerShell commands to create and start a Hyper-V VM.

        The caller is responsible for:
          - Having a sysprepped base VHDX at base_vhdx
          - Running the script as Administrator
          - Cleaning up via Remove-VM after use
        """
        name = vm_name or self.name
        mem = self.memory_mb * 1024 * 1024  # bytes for Hyper-V cmdlets
        cpu = self.cpu_count

        lines = [
            f"# Generated by bare-cua Sandboxfile: {self.name}",
            f'$VMName = "{name}"',
            f'$BaseVHDX = "{base_vhdx}"',
            f'$VMDir   = "{vm_dir}\\{name}"',
            f'$DiffVHDX = "$VMDir\\{name}-diff.vhdx"',
            f'$SwitchName = "{switch_name}"',
            "",
            "# Create differencing disk from base image (preserves base for reuse)",
            "New-Item -ItemType Directory -Force -Path $VMDir | Out-Null",
            "New-VHD -Path $DiffVHDX -ParentPath $BaseVHDX -Differencing | Out-Null",
            "",
            "# Create VM",
            f"New-VM -Name $VMName -MemoryStartupBytes {mem} -VHDPath $DiffVHDX -Generation 2 -Path $VMDir -SwitchName $SwitchName",
            f"Set-VMProcessor -VMName $VMName -Count {cpu}",
            "Set-VMFirmware -VMName $VMName -EnableSecureBoot Off",
            "",
        ]

        if self.virtual_gpu:
            lines += [
                "# GPU-PV (paravirtualized GPU — Windows 11 client only, WDDM shared kernel)",
                "# For production use, prefer Windows Server 2025 GPU-P via SR-IOV",
                "Add-VMGpuPartitionAdapter -VMName $VMName",
                "Set-VMGpuPartitionAdapter -VMName $VMName -MinPartitionVRAM 80000000 -MaxPartitionVRAM 100000000 -OptimalPartitionVRAM 100000000",
                "Set-VMGpuPartitionAdapter -VMName $VMName -MinPartitionEncode 80000000 -MaxPartitionEncode 100000000 -OptimalPartitionEncode 100000000",
                "Set-VMGpuPartitionAdapter -VMName $VMName -MinPartitionDecode 80000000 -MaxPartitionDecode 100000000 -OptimalPartitionDecode 100000000",
                "Set-VMGpuPartitionAdapter -VMName $VMName -MinPartitionCompute 80000000 -MaxPartitionCompute 100000000 -OptimalPartitionCompute 100000000",
                "",
            ]

        lines += [
            "# Start VM",
            "Start-VM -Name $VMName",
            "",
            "# Wait for VM heartbeat",
            "$deadline = (Get-Date).AddSeconds(120)",
            "while ((Get-VMIntegrationService -VMName $VMName -Name Heartbeat).PrimaryStatusDescription -ne 'OK') {",
            "    if ((Get-Date) -gt $deadline) { throw 'VM heartbeat timeout' }",
            "    Start-Sleep -Seconds 3",
            "}",
            "Write-Host 'VM_READY'",
        ]

        return "\n".join(lines)
