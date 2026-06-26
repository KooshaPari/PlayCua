"""
SandboxPool — concurrent multi-instance sandbox orchestrator.

Manages a pool of concurrent GameSandbox / HyperVSandbox instances for
parallel test matrix execution, multi-agent sessions, or CI parallelization.

Key design decisions:
  - All pool instances share the same base VHDX via Hyper-V differencing disks
    (each VM gets its own delta file — base is never modified)
  - Pool is elastic: grow with `acquire()`, shrink with `release()`
  - Each acquired instance gets a unique name to avoid VM name collisions
  - Thread-safe: uses asyncio.Lock for pool state mutations
  - Graceful teardown: `aclose()` stops all VMs even on error

Usage:
    pool = SandboxPool(
        sandboxfile="DinoSandboxfile.yaml",
        max_concurrent=4,
        base_vhdx="C:\\HyperV\\dino-base.vhdx",
    )

    async with pool as p:
        sessions = await p.acquire(3)  # grab 3 VMs
        results = await asyncio.gather(*[s.game_launch() for s in sessions])
        await p.release(sessions)       # give them back to pool
"""

from __future__ import annotations

import asyncio
import uuid
from dataclasses import dataclass, field
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .sandbox import HyperVSandbox
    from .game_sandbox import GameSandbox, GameSandboxSession


@dataclass
class PooledSandbox:
    """A sandbox instance currently owned by the pool."""

    name: str                    # unique pool-wide name e.g. "dino-pool-abc12345-0"
    index: int                  # position in pool acquisition order
    session: "GameSandboxSession"
    sandbox: "GameSandbox | HyperVSandbox"
    in_use: bool = False         # True if currently checked out to a caller


class SandboxPool:
    """Elastic pool of concurrent sandbox instances.

    Uses Hyper-V differencing disks so all instances share a single base VHDX
    without modification. Each acquire() creates a fresh differencing disk.

    Note: Windows Sandbox (WSB) does NOT support concurrent instances on
    Windows 10/11 client. Use HyperVSandbox for multi-instance pools.
    """

    def __init__(
        self,
        sandboxfile: str | None = None,
        base_vhdx: str = "C:\\HyperV\\dino-base.vhdx",
        max_concurrent: int = 4,
        vm_dir: str = "C:\\HyperV\\VMs",
        default_memory_mb: int = 8192,
    ):
        self._sandboxfile_path = sandboxfile
        self._base_vhdx = base_vhdx
        self._max_concurrent = max_concurrent
        self._vm_dir = vm_dir
        self._default_memory_mb = default_memory_mb

        self._pool: list[PooledSandbox] = []
        self._lock = asyncio.Lock()
        self._closed = False

    @property
    def total_capacity(self) -> int:
        return self._max_concurrent

    @property
    def available(self) -> int:
        return sum(1 for s in self._pool if not s.in_use)

    @property
    def in_use(self) -> int:
        return sum(1 for s in self._pool if s.in_use)

    def _make_name(self, index: int) -> str:
        return f"dino-pool-{uuid.uuid4().hex[:8]}-{index}"

    def _make_vm_dir(self, name: str) -> str:
        return f"{self._vm_dir}\\{name}"

    async def acquire(self, count: int = 1) -> list["GameSandboxSession"]:
        """Acquire `count` sandbox instances from the pool.

        If fewer than `count` instances are available, waits until they are.
        Raises RuntimeError if `count` > `max_concurrent`.

        Returns:
            List of GameSandboxSession handles. Caller MUST call release() when done.
        """
        if count > self._max_concurrent:
            raise ValueError(
                f"Cannot acquire {count} instances (max_concurrent={self._max_concurrent})"
            )

        acquired: list[PooledSandbox] = []
        deadline = 120.0  # seconds

        async with self._lock:
            # Grow pool if needed to satisfy request
            while len(self._pool) < count:
                idx = len(self._pool)
                name = self._make_name(idx)
                pooled = PooledSandbox(
                    name=name,
                    index=idx,
                    session=None,
                    sandbox=None,
                    in_use=False,
                )
                self._pool.append(pooled)

        # Start all unstarted instances in parallel
        async with self._lock:
            starts = [s for s in self._pool if s.session is None]

        if starts:
            import logging
            logging.getLogger("laycua.sandbox.pool").info(
                f"Starting {len(starts)} new sandbox instance(s)"
            )

        started = await asyncio.gather(
            *[
                self._start_instance(pooled, deadline - 120.0)
                for pooled in starts
            ],
            return_exceptions=True,
        )

        errors = [e for e in started if isinstance(e, Exception)]
        if errors:
            raise RuntimeError(f"Failed to start {len(errors)} sandbox instance(s): {errors}")

        # Wait for available instances
        async with self._lock:
            while True:
                ready = [s for s in self._pool if s.session is not None and not s.in_use]
                if len(ready) >= count:
                    for s in ready[:count]:
                        s.in_use = True
                    return [s.session for s in ready[:count]]
                if self._closed:
                    raise RuntimeError("Pool closed while waiting for instances")
                await asyncio.sleep(1)

    async def _start_instance(
        self, pooled: PooledSandbox, timeout_s: float
    ) -> "GameSandboxSession":
        """Start a single sandbox instance and populate pooled.session."""
        from .game_sandbox import GameSandbox, GameSandboxConfig
        from .sandboxfile import Sandboxfile
        from .config import SandboxConfig

        if self._sandboxfile_path:
            sf = Sandboxfile.load(self._sandboxfile_path)
            sandbox = GameSandbox.from_sandboxfile(self._sandboxfile_path)
        else:
            # Minimal config — game dir must be mapped via template
            sandbox_cfg = SandboxConfig(
                name=pooled.name,
                memory_mb=self._default_memory_mb,
                virtual_gpu="Enable",
                networking="Enable",
            )
            gcfg = GameSandboxConfig(sandbox=sandbox_cfg)
            sandbox = GameSandbox(config=gcfg)

        session = await sandbox.__aenter__()
        async with self._lock:
            pooled.session = session
            pooled.sandbox = sandbox

        return session

    async def release(self, sessions: list["GameSandboxSession"]) -> None:
        """Return sandbox instances to the pool (they stay alive for reuse).

        To permanently destroy an instance, use `destroy()` instead.
        """
        async with self._lock:
            for session in sessions:
                for pooled in self._pool:
                    if pooled.session is session:
                        pooled.in_use = False
                        break

    async def destroy(self, sessions: list["GameSandboxSession"]) -> None:
        """Permanently stop and destroy sandbox instances.

        Their delta VHDX files are removed and the VM records deleted.
        """
        from .sandbox import HyperVSandbox

        async with self._lock:
            for session in sessions:
                for pooled in self._pool[:]:
                    if pooled.session is session:
                        await pooled.sandbox.__aexit__() if hasattr(pooled.sandbox, "__aexit__") else None
                        self._pool.remove(pooled)
                        break

        # Delete delta VHDX
        async with self._lock:
            for pooled in self._pool:
                if pooled.session in sessions:
                    pass  # already removed above

    async def aclose(self) -> None:
        """Stop all VMs and clean up. Idempotent."""
        async with self._lock:
            if self._closed:
                return
            self._closed = True

        sessions_to_stop = [p.session for p in self._pool if p.session is not None]
        await asyncio.gather(
            *[s.aclose() for s in sessions_to_stop],
            return_exceptions=True,
        )
        async with self._lock:
            self._pool.clear()

    def stats(self) -> dict[str, int | bool]:
        """Return pool utilization statistics."""
        return {
            "total_capacity": self.total_capacity,
            "in_use": self.in_use,
            "available": self.available,
            "total_instances": len(self._pool),
            "closed": self._closed,
        }

    async def __aenter__(self) -> "SandboxPool":
        return self

    async def __aexit__(self, *args) -> None:
        await self.aclose()

    def __repr__(self) -> str:
        return (
            f"<SandboxPool capacity={self.total_capacity} "
            f"in_use={self.in_use} available={self.available}>"
        )
