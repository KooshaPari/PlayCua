"""
SandboxConfig — dataclasses and .wsb XML generator.

.wsb files are standard Windows Sandbox configuration XML, documented at:
https://learn.microsoft.com/en-us/windows/security/application-security/application-isolation/windows-sandbox/windows-sandbox-configure-using-wsb-file

Supported configuration elements (all available as of Windows 11 24H2):
  MemoryInMB          — integer MB; minimum enforced by OS is 2048
  VGpu                — Enable | Disable | Default  (Default = enabled on non-ARM64)
  Networking          — Enable | Disable | Default
  AudioInput          — Enable | Disable | Default
  VideoInput          — Enable | Disable | Default
  ProtectedClient     — Enable | Disable | Default  (AppContainer isolation on RDP session)
  PrinterRedirection  — Enable | Disable | Default
  ClipboardRedirection— Enable | Disable | Default
  MappedFolders       — array of (HostFolder, SandboxFolder, ReadOnly)
  LogonCommand        — single command string executed after logon
"""

from __future__ import annotations

import textwrap
import xml.etree.ElementTree as ET
from dataclasses import dataclass, field
from pathlib import Path
from typing import Literal

import yaml


TriState = Literal["Enable", "Disable", "Default"]


@dataclass
class MappedFolder:
    """A host-to-sandbox folder mapping.

    Args:
        host_folder:    Absolute path on the host (must exist before sandbox starts).
        sandbox_folder: Absolute path inside the sandbox. Created automatically if absent.
                        Defaults to the sandbox desktop if omitted.
        read_only:      Whether the sandbox can only read (not write) the folder.
    """

    host_folder: str
    sandbox_folder: str = ""
    read_only: bool = True

    def to_xml_element(self) -> ET.Element:
        elem = ET.Element("MappedFolder")
        host_elem = ET.SubElement(elem, "HostFolder")
        host_elem.text = self.host_folder
        if self.sandbox_folder:
            sb_elem = ET.SubElement(elem, "SandboxFolder")
            sb_elem.text = self.sandbox_folder
        ro_elem = ET.SubElement(elem, "ReadOnly")
        ro_elem.text = "true" if self.read_only else "false"
        return elem

    @staticmethod
    def from_dict(d: dict) -> "MappedFolder":
        return MappedFolder(
            host_folder=d["host"],
            sandbox_folder=d.get("sandbox", ""),
            read_only=bool(d.get("readonly", True)),
        )


@dataclass
class SandboxConfig:
    """Full configuration for a Windows Sandbox (.wsb) session.

    Args:
        name:                  Human-readable name (used for .wsb filename generation).
        memory_mb:             Memory in MB. OS minimum is 2048; anything lower is bumped up.
        virtual_gpu:           vGPU sharing with host GPU (DirectX via WDDM GPU-V).
                               On non-ARM64 hosts this is enabled by default.
                               NOTE: This is GPU-V (shared kernel), not GPU-P (partitioned).
                               Performance adequate for UI automation; not for heavy 3D.
        networking:            Whether the sandbox gets network access via NAT virtual switch.
                               The sandbox gets a NAT'd IP; it can reach the host via the
                               Hyper-V Default Switch gateway address (typically 172.x.x.1).
                               Loopback (127.0.0.1) does NOT cross the sandbox boundary —
                               use the gateway IP or a mapped-folder file as IPC rendezvous.
        audio_input:           Share host microphone into sandbox.
        video_input:           Share host webcam into sandbox (default disabled).
        protected_client:      Run sandbox RDP session inside AppContainer (extra isolation).
        printer_redirection:   Share host printers (default disabled).
        clipboard_redirection: Share host clipboard (default enabled).
        mapped_folders:        List of MappedFolder entries.
        startup_script:        Absolute path (on host) of a PowerShell script to execute
                               after logon. The file must be accessible via a mapped folder.
        startup_command:       Raw command string for LogonCommand. If both startup_script
                               and startup_command are set, startup_script takes precedence.
    """

    name: str
    memory_mb: int = 4096
    virtual_gpu: TriState = "Enable"
    networking: TriState = "Enable"
    audio_input: TriState = "Default"
    video_input: TriState = "Disable"
    protected_client: TriState = "Disable"
    printer_redirection: TriState = "Disable"
    clipboard_redirection: TriState = "Enable"
    mapped_folders: list[MappedFolder] = field(default_factory=list)
    startup_script: str | None = None
    startup_command: str | None = None

    # ------------------------------------------------------------------
    # XML generation
    # ------------------------------------------------------------------

    def to_wsb_xml(self) -> str:
        """Return the .wsb XML string for this configuration.

        The output is human-readable with indentation so it can be inspected
        or committed to source control alongside Sandboxfile.yaml.
        """
        root = ET.Element("Configuration")

        # Memory
        mem = ET.SubElement(root, "MemoryInMB")
        mem.text = str(max(self.memory_mb, 2048))

        # vGPU
        vgpu = ET.SubElement(root, "vGPU")
        vgpu.text = self.virtual_gpu

        # Networking
        net = ET.SubElement(root, "Networking")
        net.text = self.networking

        # Audio
        audio = ET.SubElement(root, "AudioInput")
        audio.text = self.audio_input

        # Video
        video = ET.SubElement(root, "VideoInput")
        video.text = self.video_input

        # Protected client
        pc = ET.SubElement(root, "ProtectedClient")
        pc.text = self.protected_client

        # Printer
        printer = ET.SubElement(root, "PrinterRedirection")
        printer.text = self.printer_redirection

        # Clipboard
        clip = ET.SubElement(root, "ClipboardRedirection")
        clip.text = self.clipboard_redirection

        # Mapped folders
        if self.mapped_folders:
            mf_container = ET.SubElement(root, "MappedFolders")
            for mf in self.mapped_folders:
                mf_container.append(mf.to_xml_element())

        # Logon command
        logon_cmd = self._resolve_logon_command()
        if logon_cmd:
            logon = ET.SubElement(root, "LogonCommand")
            cmd_elem = ET.SubElement(logon, "Command")
            cmd_elem.text = logon_cmd

        ET.indent(root, space="  ")
        xml_str = ET.tostring(root, encoding="unicode", xml_declaration=False)
        return f'<?xml version="1.0" encoding="utf-8"?>\n{xml_str}\n'

    def _resolve_logon_command(self) -> str | None:
        if self.startup_script:
            # The script must be inside the sandbox (via a mapped folder).
            # We wrap it in powershell to allow long-form setup scripts.
            return (
                f"powershell.exe -ExecutionPolicy Bypass -NonInteractive "
                f'-File "{self.startup_script}"'
            )
        return self.startup_command

    # ------------------------------------------------------------------
    # YAML round-trip
    # ------------------------------------------------------------------

    def to_yaml(self) -> str:
        """Serialize this config to YAML (Sandboxfile-compatible format)."""
        data: dict = {
            "name": self.name,
            "base": "windows-sandbox",
            "memory_mb": self.memory_mb,
            "virtual_gpu": self.virtual_gpu == "Enable",
            "networking": self.networking == "Enable",
            "audio_input": self.audio_input,
            "video_input": self.video_input,
            "protected_client": self.protected_client,
            "printer_redirection": self.printer_redirection,
            "clipboard_redirection": self.clipboard_redirection,
        }
        if self.mapped_folders:
            data["map"] = [
                {
                    "host": mf.host_folder,
                    "sandbox": mf.sandbox_folder,
                    "readonly": mf.read_only,
                }
                for mf in self.mapped_folders
            ]
        if self.startup_script:
            data["startup_script"] = self.startup_script
        elif self.startup_command:
            data["startup_command"] = self.startup_command
        return yaml.dump(data, default_flow_style=False, allow_unicode=True)

    @staticmethod
    def from_yaml(path: str) -> "SandboxConfig":
        """Load a SandboxConfig from a YAML file (Sandboxfile or standalone config)."""
        with open(path, "r", encoding="utf-8") as fh:
            data = yaml.safe_load(fh)
        return SandboxConfig._from_dict(data)

    @staticmethod
    def _from_dict(data: dict) -> "SandboxConfig":
        def tri(val: bool | str | None, default: TriState = "Default") -> TriState:
            if val is None:
                return default
            if isinstance(val, bool):
                return "Enable" if val else "Disable"
            return val  # type: ignore[return-value]

        mapped: list[MappedFolder] = [
            MappedFolder.from_dict(m) for m in data.get("map", [])
        ]
        return SandboxConfig(
            name=data.get("name", "unnamed-sandbox"),
            memory_mb=int(data.get("memory_mb", 4096)),
            virtual_gpu=tri(data.get("virtual_gpu"), "Enable"),
            networking=tri(data.get("networking"), "Enable"),
            audio_input=tri(data.get("audio_input"), "Default"),
            video_input=tri(data.get("video_input"), "Disable"),
            protected_client=tri(data.get("protected_client"), "Disable"),
            printer_redirection=tri(data.get("printer_redirection"), "Disable"),
            clipboard_redirection=tri(data.get("clipboard_redirection"), "Enable"),
            mapped_folders=mapped,
            startup_script=data.get("startup_script"),
            startup_command=data.get("startup_command"),
        )

    # ------------------------------------------------------------------
    # File I/O helpers
    # ------------------------------------------------------------------

    def write_wsb(self, path: str | Path | None = None) -> Path:
        """Write the .wsb file to disk.  Returns the path written."""
        if path is None:
            path = Path(f"{self.name}.wsb")
        path = Path(path)
        path.write_text(self.to_wsb_xml(), encoding="utf-8")
        return path
