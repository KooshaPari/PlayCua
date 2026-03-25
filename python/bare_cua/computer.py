"""bare_cua.computer - async Python client for bare-cua-native Rust binary.

Spawns the native binary as a subprocess and communicates over stdio using
newline-delimited JSON-RPC 2.0. No VM, no Docker, no network socket.
"""

from __future__ import annotations

import asyncio
import base64
import json
import os
import sys
from typing import Any

__all__ = ["Computer", "ComputerError"]


class ComputerError(Exception):
    """Raised when the native binary returns a JSON-RPC error response."""

    def __init__(self, code: int, message: str) -> None:
        super().__init__(f"RPC error {code}: {message}")
        self.code = code
        self.rpc_message = message


class Computer:
    """Async context manager that owns the bare-cua-native subprocess.

    Usage::

        async with Computer() as computer:
            png_bytes = await computer.screenshot()
            await computer.left_click(100, 200)
            await computer.type_text("hello world")

    Parameters
    ----------
    native_path:
        Path or name on PATH of the ``bare-cua-native`` binary.
    log_level:
        Passed as ``BARE_CUA_LOG`` env var; controls Rust tracing on stderr.
    """

    def __init__(
        self,
        native_path: str = "bare-cua-native",
        log_level: str = "info",
    ) -> None:
        self._native_path = native_path
        self._log_level = log_level
        self._proc: asyncio.subprocess.Process | None = None
        self._id: int = 0
        self._lock = asyncio.Lock()

    async def __aenter__(self) -> "Computer":
        await self._start()
        return self

    async def __aexit__(self, *_: Any) -> None:
        await self._stop()

    async def _start(self) -> None:
        env = {**os.environ, "BARE_CUA_LOG": self._log_level}
        self._proc = await asyncio.create_subprocess_exec(
            self._native_path,
            stdin=asyncio.subprocess.PIPE,
            stdout=asyncio.subprocess.PIPE,
            stderr=sys.stderr,
            env=env,
        )
        ok = await self.ping()
        if not ok:
            raise RuntimeError("bare-cua-native did not respond to ping")

    async def _stop(self) -> None:
        if self._proc is not None:
            try:
                if self._proc.stdin and not self._proc.stdin.is_closing():
                    self._proc.stdin.close()
                    await self._proc.stdin.wait_closed()
            except Exception:
                pass
            try:
                await asyncio.wait_for(self._proc.wait(), timeout=3.0)
            except asyncio.TimeoutError:
                self._proc.kill()
            self._proc = None

    async def _call(self, method: str, **params: Any) -> Any:
        """Send a JSON-RPC 2.0 request and return the result field."""
        if self._proc is None:
            raise RuntimeError("Computer not started - use async with Computer()")
        async with self._lock:
            self._id += 1
            req_id = self._id
            request = {
                "jsonrpc": "2.0",
                "id": req_id,
                "method": method,
                "params": params or {},
            }
            line = json.dumps(request, separators=(",", ":")) + "\n"
            assert self._proc.stdin is not None
            self._proc.stdin.write(line.encode())
            await self._proc.stdin.drain()
            assert self._proc.stdout is not None
            raw = await self._proc.stdout.readline()
            if not raw:
                raise RuntimeError("bare-cua-native closed stdout unexpectedly")
            resp = json.loads(raw.decode())
        if resp.get("error"):
            err = resp["error"]
            raise ComputerError(err.get("code", -1), err.get("message", "unknown"))
        return resp.get("result")

    # Screenshot

    async def screenshot(self, window_title: str | None = None, monitor: int = 0) -> bytes:
        """Capture a screenshot. Returns raw PNG bytes."""
        result = await self._call("screenshot", window_title=window_title, monitor=monitor)
        return base64.b64decode(result["data"])

    # Mouse

    async def left_click(self, x: int, y: int) -> None:
        await self._call("input.click", x=x, y=y, button="left", action="click")

    async def right_click(self, x: int, y: int) -> None:
        await self._call("input.click", x=x, y=y, button="right", action="click")

    async def double_click(self, x: int, y: int) -> None:
        await self._call("input.click", x=x, y=y, button="left", action="click")
        await asyncio.sleep(0.05)
        await self._call("input.click", x=x, y=y, button="left", action="click")

    async def middle_click(self, x: int, y: int) -> None:
        await self._call("input.click", x=x, y=y, button="middle", action="click")

    async def mouse_down(self, x: int, y: int, button: str = "left") -> None:
        await self._call("input.click", x=x, y=y, button=button, action="down")

    async def mouse_up(self, x: int, y: int, button: str = "left") -> None:
        await self._call("input.click", x=x, y=y, button=button, action="up")

    async def move_mouse(self, x: int, y: int) -> None:
        await self._call("input.move", x=x, y=y)

    async def scroll(self, x: int, y: int, direction: str = "down", amount: int = 3) -> None:
        await self._call("input.scroll", x=x, y=y, direction=direction, amount=amount)

    # Keyboard

    async def type_text(self, text: str) -> None:
        await self._call("input.type", text=text)

    async def press_key(self, key: str) -> None:
        await self._call("input.key", key=key, action="press")

    async def key_down(self, key: str) -> None:
        await self._call("input.key", key=key, action="down")

    async def key_up(self, key: str) -> None:
        await self._call("input.key", key=key, action="up")

    # Window management

    async def list_windows(self) -> list[dict]:
        """Return a list of all top-level windows."""
        return await self._call("windows.list")  # type: ignore[return-value]

    async def find_window(self, title: str | None = None, pid: int | None = None) -> dict | None:
        """Find a window by partial title or PID. Returns None if not found."""
        return await self._call("windows.find", title=title, pid=pid)

    async def focus_window(self, hwnd: int) -> None:
        """Bring a window to the foreground by HWND."""
        await self._call("windows.focus", hwnd=hwnd)

    # Process management

    async def launch_process(
        self, path: str, args: list[str] | None = None, cwd: str | None = None
    ) -> int:
        """Launch a process non-blocking. Returns the PID."""
        result = await self._call("process.launch", path=path, args=args or [], cwd=cwd)
        return result["pid"]

    async def kill_process(self, pid: int) -> None:
        """Kill a process by PID."""
        await self._call("process.kill", pid=pid)

    async def process_status(self, pid: int) -> dict:
        """Return running state and optional exit code."""
        return await self._call("process.status", pid=pid)  # type: ignore[return-value]

    # Image analysis

    async def frames_differ(
        self, image_a: bytes, image_b: bytes, threshold: float = 0.02
    ) -> bool:
        """Return True if images differ by more than threshold fraction of pixels."""
        result = await self._call(
            "analysis.diff",
            image_a=base64.b64encode(image_a).decode(),
            image_b=base64.b64encode(image_b).decode(),
            threshold=threshold,
        )
        return result["changed"]

    async def image_hash(self, image: bytes) -> str:
        """Return BLAKE3 hex hash of the image pixel data."""
        result = await self._call("analysis.hash", image=base64.b64encode(image).decode())
        return result["hash"]

    # Utility

    async def ping(self) -> bool:
        """Return True if the native binary is alive and responding."""
        try:
            result = await self._call("ping")
            return bool(result.get("ok", False))
        except Exception:
            return False

    async def wait_for_visual_change(
        self,
        timeout: float = 30.0,
        poll_interval: float = 0.5,
        threshold: float = 0.02,
        window_title: str | None = None,
    ) -> bytes:
        """Poll screenshots until a visual change is detected.

        Returns the first screenshot that differs from the baseline.
        Raises TimeoutError if no change occurs within timeout seconds.
        """
        baseline = await self.screenshot(window_title=window_title)
        deadline = asyncio.get_event_loop().time() + timeout
        while asyncio.get_event_loop().time() < deadline:
            await asyncio.sleep(poll_interval)
            current = await self.screenshot(window_title=window_title)
            if await self.frames_differ(baseline, current, threshold=threshold):
                return current
        raise TimeoutError(f"No visual change detected within {timeout}s")
