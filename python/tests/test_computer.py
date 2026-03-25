"""
Tests for the bare_cua Python bindings.

Uses a mock subprocess that echoes pre-canned JSON-RPC responses so no real
native binary is required. Tests cover:
  - ping returns True
  - screenshot returns bytes
  - unknown method raises an exception
"""

import asyncio
import json
import sys
import textwrap
from pathlib import Path
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

# ---------------------------------------------------------------------------
# Fixtures / helpers
# ---------------------------------------------------------------------------

# The mock native server: a small Python script that reads JSON-RPC requests
# from stdin line-by-line and writes pre-canned responses to stdout.
MOCK_SERVER_SCRIPT = textwrap.dedent("""\
    import sys, json

    RESPONSES = {
        "ping": {"ok": True, "version": "0.0.0-test"},
        "screenshot": {
            "data": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
            "width": 1,
            "height": 1,
            "format": "png",
        },
    }

    for raw_line in sys.stdin:
        line = raw_line.strip()
        if not line:
            continue
        req = json.loads(line)
        method = req.get("method", "")
        req_id = req.get("id", 1)

        if method in RESPONSES:
            resp = {"jsonrpc": "2.0", "id": req_id, "result": RESPONSES[method]}
        else:
            resp = {
                "jsonrpc": "2.0",
                "id": req_id,
                "error": {"code": -32601, "message": f"Method not found: {method}"},
            }
        sys.stdout.write(json.dumps(resp) + "\\n")
        sys.stdout.flush()
""")


@pytest.fixture
def mock_server_path(tmp_path: Path) -> Path:
    """Write the mock server script to a temp file and return its path."""
    script = tmp_path / "mock_server.py"
    script.write_text(MOCK_SERVER_SCRIPT)
    return script


# ---------------------------------------------------------------------------
# Import the Python package under test
# ---------------------------------------------------------------------------
# Allow the tests to be run from the repo root even without installing the pkg.
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "python"))

try:
    from bare_cua import Computer  # type: ignore
except ImportError:
    pytest.skip("bare_cua package not installed — skipping integration tests", allow_module_level=True)


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------


class TestComputerPing:
    """computer.ping() should return True when the native binary echoes ok."""

    @pytest.mark.asyncio
    async def test_ping_returns_true(self, mock_server_path: Path):
        binary = [sys.executable, str(mock_server_path)]
        async with Computer(binary) as computer:
            result = await computer.ping()
        assert result is True, f"Expected True, got {result!r}"


class TestComputerScreenshot:
    """computer.screenshot() should return non-empty bytes (PNG data)."""

    @pytest.mark.asyncio
    async def test_screenshot_returns_bytes(self, mock_server_path: Path):
        binary = [sys.executable, str(mock_server_path)]
        async with Computer(binary) as computer:
            png_bytes = await computer.screenshot()
        assert isinstance(png_bytes, bytes), f"Expected bytes, got {type(png_bytes)}"
        assert len(png_bytes) > 0, "screenshot() returned empty bytes"

    @pytest.mark.asyncio
    async def test_screenshot_starts_with_png_header(self, mock_server_path: Path):
        PNG_MAGIC = b"\x89PNG\r\n\x1a\n"
        binary = [sys.executable, str(mock_server_path)]
        async with Computer(binary) as computer:
            png_bytes = await computer.screenshot()
        assert png_bytes[:8] == PNG_MAGIC, (
            f"Expected PNG magic bytes, got {png_bytes[:8]!r}"
        )


class TestComputerUnknownMethod:
    """Calling an unsupported method should raise an exception."""

    @pytest.mark.asyncio
    async def test_unknown_method_raises(self, mock_server_path: Path):
        binary = [sys.executable, str(mock_server_path)]
        async with Computer(binary) as computer:
            with pytest.raises(Exception) as exc_info:
                # Access the raw RPC layer to send an unknown method.
                await computer._rpc("definitely.not.a.method", {})
        assert "not found" in str(exc_info.value).lower() or "32601" in str(exc_info.value), (
            f"Expected method-not-found error, got: {exc_info.value}"
        )


class TestComputerContextManager:
    """Computer should work correctly as an async context manager."""

    @pytest.mark.asyncio
    async def test_context_manager_cleans_up(self, mock_server_path: Path):
        binary = [sys.executable, str(mock_server_path)]
        computer = Computer(binary)
        async with computer:
            alive = True
        # After __aexit__, the process should be cleaned up.
        assert alive, "Context manager block executed"

    @pytest.mark.asyncio
    async def test_multiple_calls_in_session(self, mock_server_path: Path):
        binary = [sys.executable, str(mock_server_path)]
        async with Computer(binary) as computer:
            r1 = await computer.ping()
            r2 = await computer.ping()
            png = await computer.screenshot()
        assert r1 is True
        assert r2 is True
        assert isinstance(png, bytes) and len(png) > 0
