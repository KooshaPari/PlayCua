from __future__ import annotations
import asyncio, json, os, subprocess, sys, tempfile, time
from dataclasses import dataclass
from pathlib import Path
from typing import Any
from .config import MappedFolder, SandboxConfig
from .sandboxfile import Sandboxfile

@dataclass
class CommandResult:
    command: str
    exit_code: int
    output: str
    error: str
    @property
    def success(self) -> bool: return self.exit_code == 0

class Computer:
    def __init__(self, sandbox_ip=None, native_port=8765, shared_dir=None, proc=None):
        self._sandbox_ip = sandbox_ip
        self._native_port = native_port
        self._shared_dir = Path(shared_dir) if shared_dir else None
        self._proc = proc
        self._mode = "tcp" if sandbox_ip else "file"

    @property
    def sandbox_ip(self): return self._sandbox_ip

    async def run(self, command: str, timeout_s: int = 300):
        if self._mode == "tcp": return await self._run_tcp(command, timeout_s)
        return await self._run_file_drop(command, timeout_s)

    async def _run_tcp(self, command: str, timeout_s: int):
        if not self._sandbox_ip: raise RuntimeError("No sandbox IP")
        req = json.dumps({"type": "run", "command": command}) + chr(10)
        try:
            r, w = await asyncio.wait_for(
                asyncio.open_connection(self._sandbox_ip, self._native_port), timeout=10)
            w.write(req.encode()); await w.drain()
            raw = await asyncio.wait_for(r.readline(), timeout=timeout_s)
            w.close(); await w.wait_closed()
            resp = json.loads(raw.decode())
            return CommandResult(command=command, exit_code=resp.get("exit_code",-1),
                output=resp.get("stdout",""), error=resp.get("stderr",""))
        except (asyncio.TimeoutError, ConnectionRefusedError, OSError) as e:
            return CommandResult(command=command, exit_code=-1, output="", error=str(e))

    async def _run_file_drop(self, command: str, timeout_s: int):
        if not self._shared_dir: raise RuntimeError("No shared_dir")
        rid = str(int(time.time() * 1000))
        rq = self._shared_dir / f"cmd_{rid}.json"
        rs = self._shared_dir / f"res_{rid}.json"
        rq.write_text(json.dumps({"id": rid, "command": command}), encoding="utf-8")
        dl = time.monotonic() + timeout_s
        while time.monotonic() < dl:
            if rs.exists():
                resp = json.loads(rs.read_text(encoding="utf-8"))
                try: rq.unlink(missing_ok=True); rs.unlink(missing_ok=True)
                except OSError: pass
                return CommandResult(command=command, exit_code=resp.get("exit_code",-1),
                    output=resp.get("stdout",""), error=resp.get("stderr",""))
            await asyncio.sleep(0.5)
        return CommandResult(command=command, exit_code=-1, output="",
                             error=f"Timeout after {timeout_s}s")

    def is_alive(self) -> bool:
        return True if self._proc is None else self._proc.poll() is None

    def terminate(self) -> None:
        if self._proc and self._proc.poll() is None:
            self._proc.terminate()
            try: self._proc.wait(timeout=10)
            except subprocess.TimeoutExpired: self._proc.kill()


class Sandbox:
    SANDBOX_EXE: str = r"C:\Windows\System32\WindowsSandbox.exe"
    TEMP_DIR = Path(os.environ.get("TEMP", tempfile.gettempdir())) / "bare-cua-sandbox"

    def __init__(self, config, sandboxfile=None, native_port=8765, poll_interval_s=2.0):
        self._config = config; self._sandboxfile = sandboxfile
        self._native_port = native_port; self._poll_interval = poll_interval_s
        self._proc = None; self._wsb_path = None
        self._script_path = None; self._shared_dir = None; self._computer = None

    @classmethod
    def from_sandboxfile(cls, path, native_port=8765):
        sf = Sandboxfile.load(path)
        return cls(config=sf.to_sandbox_config(), sandboxfile=sf, native_port=native_port)

    @classmethod
    def from_config(cls, config, **kw): return cls(config=config, **kw)

    async def __aenter__(self):
        self.TEMP_DIR.mkdir(parents=True, exist_ok=True)
        self._shared_dir = self.TEMP_DIR / "shared"
        self._shared_dir.mkdir(exist_ok=True)
        shared_mf = MappedFolder(host_folder=str(self._shared_dir),
            sandbox_folder=r"C:\SandboxShared", read_only=False)
        config = self._config; extra = [shared_mf]
        startup_cmd = config.startup_command
        if self._sandboxfile is not None:
            sc = self._sandboxfile.render_startup_script(
                ready_flag_path=r"C:\SandboxShared\ready.flag")
            self._script_path = self.TEMP_DIR / "setup.ps1"
            self._script_path.write_text(sc, encoding="utf-8")
            temp_mf = MappedFolder(host_folder=str(self.TEMP_DIR),
                sandbox_folder=r"C:\SandboxInit", read_only=True)
            extra.append(temp_mf)
            startup_cmd = (r"powershell.exe -ExecutionPolicy Bypass -NonInteractive"
                           r" -File C:\SandboxInit\setup.ps1")
        config = SandboxConfig(
            name=config.name, memory_mb=config.memory_mb,
            virtual_gpu=config.virtual_gpu, networking=config.networking,
            audio_input=config.audio_input, video_input=config.video_input,
            protected_client=config.protected_client,
            printer_redirection=config.printer_redirection,
            clipboard_redirection=config.clipboard_redirection,
            mapped_folders=[*config.mapped_folders, *extra],
            startup_command=startup_cmd,
        )
        self._wsb_path = self.TEMP_DIR / f"{config.name}.wsb"
        config.write_wsb(self._wsb_path)
        self._proc = self._launch_sandbox(self._wsb_path)
        await asyncio.sleep(5)
        hc = self._sandboxfile.health_check if self._sandboxfile else None
        timeout_s = hc.timeout_s if hc else 120
        ip = await self._wait_for_ready(timeout_s)
        self._computer = Computer(sandbox_ip=ip, native_port=self._native_port,
            shared_dir=str(self._shared_dir), proc=self._proc)
        return self._computer

    async def __aexit__(self, *a):
        if self._computer: self._computer.terminate()
        for p in [self._wsb_path, self._script_path]:
            if p and p.exists():
                try: p.unlink()
                except OSError: pass

    def _launch_sandbox(self, wsb_path):
        if not Path(self.SANDBOX_EXE).exists():
            raise EnvironmentError(f"WindowsSandbox.exe not found: {self.SANDBOX_EXE}")
        return subprocess.Popen([self.SANDBOX_EXE, str(wsb_path)],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    async def _wait_for_ready(self, timeout_s):
        rf = self._shared_dir / "ready.flag"
        ef = self._shared_dir / "ready.flag.error"
        ipf = self._shared_dir / "sandbox_ip.txt"
        dl = time.monotonic() + timeout_s
        while time.monotonic() < dl:
            if ef.exists(): raise RuntimeError(f"Setup failed: {ef.read_text()}")
            if rf.exists():
                return ipf.read_text(encoding="utf-8").strip() if ipf.exists() else None
            if self._proc and self._proc.poll() is not None:
                raise RuntimeError(f"WSB exited (code={self._proc.returncode})")
            await asyncio.sleep(self._poll_interval)
        raise TimeoutError(f"Not ready within {timeout_s}s. Flag: {rf}")

    async def run_tests(self):
        if self._computer is None: raise RuntimeError('Must be in context')
        if self._sandboxfile is None: raise RuntimeError('No Sandboxfile')
        results = []
        for cmd in self._sandboxfile.test_commands:
            r = await self._computer.run(cmd)
            results.append(r)
            status = 'PASS' if r.success else 'FAIL'
            print(f"[{status}] {cmd}")
            if r.output: print(r.output)
            if r.error: print(f"STDERR: {r.error}", file=sys.stderr)
        return results


class HyperVSandbox:
    """Hyper-V VM sandbox -- multiple concurrent instances, full GPU support.

    GPU options:
      Win11 client:        Add-VMGpuPartitionAdapter (GPU-PV, WDDM shared kernel)
      Windows Server 2025: GPU-P via SR-IOV (hardware-isolated GPU slices)
      DDA:                 entire GPU dedicated to one VM (server hw + iommu)
    """

    def __init__(self, sandboxfile, base_vhdx, vm_dir=r"C:\HyperV\VMs",
                 switch_name="Default Switch"):
        self._sf = sandboxfile; self._base_vhdx = base_vhdx
        self._vm_dir = vm_dir; self._switch_name = switch_name; self._vm_name = None

    async def __aenter__(self):
        import uuid
        self._vm_name = f"{self._sf.name}-{uuid.uuid4().hex[:8]}"
        await self._run_ps(self._sf.to_hyperv_script(
            vm_name=self._vm_name, base_vhdx=self._base_vhdx,
            vm_dir=self._vm_dir, switch_name=self._switch_name))
        return self

    async def __aexit__(self, *_):
        if self._vm_name:
            cmds = [f"Stop-VM -Name {chr(39)}{self._vm_name}{chr(39)} -Force -TurnOff",
                    f"Remove-VM -Name {chr(39)}{self._vm_name}{chr(39)} -Force"]
            await self._run_ps(chr(10).join(cmds))

    async def _run_ps(self, script: str) -> str:
        """Invoke PowerShell via list-form exec (no shell injection)."""
        proc = await asyncio.create_subprocess_exec(
            "powershell.exe", "-NonInteractive", "-ExecutionPolicy", "Bypass",
            "-Command", script,
            stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE)
        so, se = await proc.communicate()
        if proc.returncode != 0:
            raise RuntimeError(f"PS error (exit={proc.returncode}): {se.decode()}")
        return so.decode("utf-8", errors="replace")

    async def get_ip(self) -> str | None:
        if not self._vm_name: return None
        name = self._vm_name
        script = (f"(Get-VMNetworkAdapter -VMName {chr(39)}{name}{chr(39)}).IPAddresses"
                  " | Where-Object { $_ -match chr(39)^[0-9]chr(39) }"
                  " | Select-Object -First 1")
        return (await self._run_ps(script)).strip() or None
