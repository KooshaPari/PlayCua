"""
SteamlessHelper — strip Steam DRM from DINO executable + manage base image lifecycle.

This module handles:
  1. Downloading Steamless if not present
  2. Stripping PackedFile0.dun from the DINO executable
  3. Creating a "base image" directory — a clean, stripped copy of the game
     ready for use as a mapped folder in sandbox VMs
  4. Validating that the stripped executable runs without Steam

The base image is a plain directory copy (not compressed archive).
For sandbox use, map the base image directory directly into the VM — no extraction step.
This avoids the ~30-60s decompression overhead on each cold start.

Usage:
    helper = SteamlessHelper(
        source_game_dir="G:\\SteamLibrary\\steamapps\\common\\Diplomacy is Not an Option",
        base_image_dir="C:\\DINOForge\\base_images\\dino-stripped",
    )

    # Strip once (first time only):
    if not helper.is_stripped():
        helper.download_steamless()
        helper.strip()

    # Copy to target dir for sandbox use (fast: just file copy, not re-strip):
    helper.clone_to("C:\\HyperV\\BaseImages\\dino-run-001")

    # Or map the base image directly into a SandboxConfig:
    sandbox_cfg = helper.to_sandbox_config("C:\\DINO")

Steamless downloads from: https://github.com/atom0s/Steamless/releases
"""

from __future__ import annotations

import os
import shutil
import subprocess
import urllib.request
import zipfile
from pathlib import Path
from dataclasses import dataclass


STEAMLESS_VERSION = "0.3.1"
STEAMLESS_URL = (
    f"https://github.com/atom0s/Steamless/releases/download/v{STEAMLESS_VERSION}/"
    f"Steamless.v{STEAMLESS_VERSION}.zip"
)

DINO_APP_ID = "1273720"


@dataclass
class SteamlessResult:
    success: bool
    message: str
    stripped_files: list[str]
    errors: list[str]


class SteamlessHelper:
    """Steamless DRM stripper + base image manager for DINOForge sandboxing."""

    def __init__(
        self,
        source_game_dir: str,
        base_image_dir: str,
        steamless_dir: str | None = None,
    ):
        self.source_game_dir = Path(source_game_dir)
        self.base_image_dir = Path(base_image_dir)
        self.steamless_dir = Path(steamless_dir or str(Path.home() / ".laycua" / "steamless"))
        self.steamless_exe = self.steamless_dir / "Steamless.CLI.exe"

        self._game_exe = self.source_game_dir / "Diplomacy is Not an Option.exe"
        self._stripped_exe = self.base_image_dir / "Diplomacy is Not an Option.exe"

    # -------------------------------------------------------------------------
    # Steamless binary management
    # -------------------------------------------------------------------------

    def get_steamless_version(self) -> str | None:
        """Return the installed Steamless version, or None if not installed."""
        if not self.steamless_exe.exists():
            return None
        try:
            result = subprocess.run(
                [str(self.steamless_exe), "--version"],
                capture_output=True,
                text=True,
                timeout=10,
            )
            return result.stdout.strip() or STEAMLESS_VERSION
        except (subprocess.TimeoutExpired, OSError):
            return None

    def download_steamless(self, force: bool = False) -> Path:
        """Download and extract Steamless to steamless_dir.

        Returns the path to Steamless.CLI.exe.

        Raises:
            RuntimeError: if download or extraction fails.
        """
        if self.steamless_exe.exists() and not force:
            return self.steamless_exe

        self.steamless_dir.mkdir(parents=True, exist_ok=True)
        zip_path = self.steamless_dir / f"steamless-{STEAMLESS_VERSION}.zip"

        print(f"[SteamlessHelper] Downloading Steamless v{STEAMLESS_VERSION}...")
        try:
            urllib.request.urlretrieve(STEAMLESS_URL, zip_path)
        except OSError as e:
            raise RuntimeError(f"Failed to download Steamless: {e}") from e

        print(f"[SteamlessHelper] Extracting to {self.steamless_dir}...")
        try:
            with zipfile.ZipFile(zip_path, "r") as zf:
                zf.extractall(self.steamless_dir)
        except zipfile.BadZipFile as e:
            raise RuntimeError(f"Invalid Steamless zip: {e}") from e
        finally:
            zip_path.unlink(missing_ok=True)

        if not self.steamless_exe.exists():
            # Steamless v0.3.x may use a different filename — find it
            candidates = list(self.steamless_dir.glob("**/*.exe"))
            if candidates:
                self.steamless_exe = candidates[0]
            else:
                raise RuntimeError(
                    f"Steamless.CLI.exe not found after extraction in {self.steamless_dir}. "
                    f"Contents: {[p.name for p in self.steamless_dir.iterdir()]}"
                )

        return self.steamless_exe

    # -------------------------------------------------------------------------
    # DRM stripping
    # -------------------------------------------------------------------------

    def is_stripped(self) -> bool:
        """Check whether the source game executable has been stripped.

        A stripped executable has no 'PackedFile0.dun' overlay.
        """
        if not self._game_exe.exists():
            return False
        try:
            result = subprocess.run(
                [str(self.steamless_exe), str(self._game_exe), "--info"],
                capture_output=True,
                text=True,
                timeout=30,
            )
            return "protected" not in result.stdout.lower()
        except OSError:
            return False

    def strip(self) -> SteamlessResult:
        """Strip Steam DRM from the DINO executable.

        The stripped executable is written to base_image_dir.
        The source executable is NOT modified.

        Returns:
            SteamlessResult with success status, message, and affected files.

        Raises:
            RuntimeError: if Steamless binary is not found (call download_steamless first).
        """
        if not self.steamless_exe.exists():
            raise RuntimeError(
                "Steamless not found. Call download_steamless() first."
            )

        self.base_image_dir.mkdir(parents=True, exist_ok=True)

        # Copy entire game directory to base_image_dir first
        print(f"[SteamlessHelper] Copying game files to {self.base_image_dir}...")
        _copytree(self.source_game_dir, self.base_image_dir, dirs_exist_ok=False)

        # Strip the copy in-place
        stripped_exe = self.base_image_dir / "Diplomacy is Not an Option.exe"
        print(f"[SteamlessHelper] Stripping DRM from {stripped_exe}...")

        try:
            result = subprocess.run(
                [str(self.steamless_exe), str(stripped_exe), "--extract"],
                capture_output=True,
                text=True,
                timeout=120,
            )
        except subprocess.TimeoutExpired as e:
            return SteamlessResult(
                success=False,
                message="Steamless timed out after 120s",
                stripped_files=[],
                errors=[str(e)],
            )

        errors: list[str] = []
        if result.returncode != 0:
            errors.append(f"Steamless exit code {result.returncode}: {result.stderr}")

        # Steamless v0.3.x extracts to a .unpacked subdir
        unpacked_dir = stripped_exe.with_suffix(".exe.unpacked")
        if unpacked_dir.exists():
            unpacked_exe = unpacked_dir / "Diplomacy is Not an Option.exe"
            if unpacked_exe.exists():
                shutil.copy2(unpacked_exe, stripped_exe)
                shutil.rmtree(unpacked_dir, ignore_errors=True)

        return SteamlessResult(
            success=result.returncode == 0 and not errors,
            message=result.stdout.strip() or "Strip complete",
            stripped_files=[str(stripped_exe)],
            errors=errors,
        )

    # -------------------------------------------------------------------------
    # Base image management
    # -------------------------------------------------------------------------

    def clone_to(self, target_dir: str) -> Path:
        """Clone the base image to a target directory for sandbox use.

        This is a fast directory copy — NO re-stripping, NO extraction.
        The target directory is created if it doesn't exist.

        Args:
            target_dir: Absolute path where the cloned image should live.

        Returns:
            Path to the cloned game exe.
        """
        target = Path(target_dir)
        if not self.base_image_dir.exists():
            raise FileNotFoundError(
                f"Base image not found at {self.base_image_dir}. "
                "Call strip() first."
            )
        target.mkdir(parents=True, exist_ok=True)
        _copytree(self.base_image_dir, target, dirs_exist_ok=False)
        return target / "Diplomacy is Not an Option.exe"

    def to_mapped_folder(
        self, sandbox_path: str = "C:\\DINO"
    ) -> "MappedFolder":
        """Return a MappedFolder pointing the base image into sandbox VMs.

        This is the recommended way to use a pre-stripped base image:
          sandbox_cfg = SandboxConfig(
              name="dino-test",
              mapped_folders=[helper.to_mapped_folder()],
          )
        """
        from .config import MappedFolder
        return MappedFolder(
            host_folder=str(self.base_image_dir),
            sandbox_folder=sandbox_path,
            read_only=True,
        )

    def verify_stripped_exe(self) -> bool:
        """Verify the stripped executable exists and has a reasonable size."""
        if not self._stripped_exe.exists():
            return False
        size_mb = self._stripped_exe.stat().st_size / (1024 * 1024)
        return 0.1 < size_mb < 500  # sanity check: not empty, not absurdly large


def _copytree(src: Path, dst: Path, dirs_exist_ok: bool) -> None:
    """Robust directory copy using shutil.

    Filters out Steam's .nomedia and large shader cache files to speed up copy.
    """
    ignores = {".nomedia", "shader_cache", "steam_autocloud.vdf"}
    for src_path in src.rglob("*"):
        if src_path.is_file():
            rel = src_path.relative_to(src)
            if any(part in ignores for part in rel.parts):
                continue
            dst_path = dst / rel
            dst_path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(src_path, dst_path)
