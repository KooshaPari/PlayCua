"""
Tests for laycua.sandbox — config, sandboxfile, GameSandbox, SandboxPool, SteamlessHelper.
"""

from __future__ import annotations

import tempfile
import xml.etree.ElementTree as ET
from pathlib import Path

import pytest

from laycua.sandbox import (
    GameSandboxConfig,
    HealthCheck,
    MappedFolder,
    SandboxConfig,
    Sandboxfile,
    SteamlessHelper,
    SteamlessResult,
)


# ---------------------------------------------------------------------------
# SandboxConfig tests
# ---------------------------------------------------------------------------

class TestSandboxConfig:
    def test_defaults(self):
        cfg = SandboxConfig(name="test-sandbox")
        assert cfg.name == "test-sandbox"
        assert cfg.memory_mb == 4096
        assert cfg.virtual_gpu == "Enable"
        assert cfg.networking == "Enable"

    def test_wsb_xml_minimal(self):
        cfg = SandboxConfig(name="minimal", memory_mb=4096)
        xml = cfg.to_wsb_xml()
        root = ET.fromstring(xml)
        assert root.tag == "Configuration"
        assert root.find("MemoryInMB").text == "4096"
        assert root.find("vGPU").text == "Enable"
        assert root.find("Networking").text == "Enable"

    def test_wsb_xml_with_mapped_folder(self):
        cfg = SandboxConfig(
            name="with-mf",
            memory_mb=8192,
            mapped_folders=[
                MappedFolder(
                    host_folder="G:\\SteamLibrary",
                    sandbox_folder="C:\\DINO",
                    read_only=True,
                ),
            ],
        )
        xml = cfg.to_wsb_xml()
        root = ET.fromstring(xml)
        mf_elem = root.find("MappedFolders/MappedFolder")
        assert mf_elem is not None
        assert mf_elem.find("HostFolder").text == "G:\\SteamLibrary"
        assert mf_elem.find("SandboxFolder").text == "C:\\DINO"
        assert mf_elem.find("ReadOnly").text == "true"

    def test_wsb_xml_with_startup_script(self):
        cfg = SandboxConfig(
            name="with-script",
            startup_script=r"C:\SandboxInit\setup.ps1",
        )
        xml = cfg.to_wsb_xml()
        root = ET.fromstring(xml)
        logon = root.find("LogonCommand/Command")
        assert logon is not None
        assert "setup.ps1" in logon.text

    def test_write_wsb(self, tmp_path):
        cfg = SandboxConfig(name="write-test")
        wsb_path = tmp_path / "test.wsb"
        written = cfg.write_wsb(wsb_path)
        assert written.exists()
        assert written.read_text(encoding="utf-8").startswith('<?xml version="1.0"')

    def test_to_yaml(self):
        cfg = SandboxConfig(
            name="yaml-test",
            memory_mb=8192,
            virtual_gpu="Enable",
            networking="Disable",
            mapped_folders=[
                MappedFolder(host_folder="G:\\Games", sandbox_folder="C:\\Games", read_only=True),
            ],
        )
        yaml_str = cfg.to_yaml()
        assert "yaml-test" in yaml_str
        assert "8192" in yaml_str

    def test_roundtrip_yaml(self, tmp_path):
        cfg = SandboxConfig(
            name="roundtrip",
            memory_mb=6144,
            virtual_gpu="Disable",
            networking="Enable",
            mapped_folders=[
                MappedFolder(host_folder="H:\\Test", sandbox_folder="C:\\T", read_only=False),
            ],
        )
        yaml_path = tmp_path / "roundtrip.yaml"
        yaml_path.write_text(cfg.to_yaml(), encoding="utf-8")
        loaded = SandboxConfig.from_yaml(yaml_path)
        assert loaded.name == cfg.name
        assert loaded.memory_mb == cfg.memory_mb
        assert loaded.virtual_gpu == cfg.virtual_gpu
        assert len(loaded.mapped_folders) == 1
        assert loaded.mapped_folders[0].host_folder == "H:\\Test"


# ---------------------------------------------------------------------------
# MappedFolder tests
# ---------------------------------------------------------------------------

class TestMappedFolder:
    def test_xml_element(self):
        mf = MappedFolder(
            host_folder="G:\\SteamLibrary",
            sandbox_folder="C:\\DINO",
            read_only=True,
        )
        elem = mf.to_xml_element()
        assert elem.tag == "MappedFolder"
        assert elem.find("HostFolder").text == "G:\\SteamLibrary"
        assert elem.find("SandboxFolder").text == "C:\\DINO"
        assert elem.find("ReadOnly").text == "true"

    def test_from_dict(self):
        d = {"host": "D:\\Games", "sandbox": "C:\\Games", "readonly": False}
        mf = MappedFolder.from_dict(d)
        assert mf.host_folder == "D:\\Games"
        assert mf.sandbox_folder == "C:\\Games"
        assert mf.read_only is False

    def test_readonly_defaults_true(self):
        mf = MappedFolder.from_dict({"host": "X:\\"})
        assert mf.read_only is True


# ---------------------------------------------------------------------------
# Sandboxfile tests
# ---------------------------------------------------------------------------

class TestSandboxfile:
    def test_load_minimal(self, tmp_path):
        yaml_content = """
name: minimal-sandbox
base: windows-sandbox
memory_mb: 4096
virtual_gpu: true
networking: true
map:
  - host: "G:/SteamLibrary/common/DINO"
    sandbox: "C:/DINO"
    readonly: true
health_check:
  type: file
  path: "C:/ready.flag"
  timeout_s: 60
"""
        sf_path = tmp_path / "Sandboxfile.yaml"
        sf_path.write_text(yaml_content, encoding="utf-8")
        sf = Sandboxfile.load(sf_path)
        assert sf.name == "minimal-sandbox"
        assert sf.base == "windows-sandbox"
        assert sf.memory_mb == 4096
        assert sf.virtual_gpu is True
        assert len(sf.mapped_folders) == 1
        assert sf.health_check.type == "file"
        assert sf.health_check.path == "C:/ready.flag"
        assert sf.health_check.timeout_s == 60

    def test_load_with_setup_steps(self, tmp_path):
        yaml_content = """
name: setup-sandbox
base: windows-sandbox
setup:
  - run: "powershell -c 'Write-Host hello'"
  - wait_for_file: "C:/BepInEx/LogOutput.log"
    timeout_s: 90
  - wait_for_process: "Diplomacy is Not an Option"
    timeout_s: 30
"""
        sf_path = tmp_path / "sf.yaml"
        sf_path.write_text(yaml_content, encoding="utf-8")
        sf = Sandboxfile.load(sf_path)
        assert len(sf.setup_steps) == 3
        assert sf.setup_steps[0].command == "powershell -c 'Write-Host hello'"
        assert sf.setup_steps[1].path == "C:/BepInEx/LogOutput.log"
        assert sf.setup_steps[1].timeout_s == 90
        assert sf.setup_steps[2].process_name == "Diplomacy is Not an Option"

    def test_to_sandbox_config(self, tmp_path):
        yaml_content = """
name: convert-test
base: windows-sandbox
memory_mb: 8192
virtual_gpu: false
networking: true
"""
        sf_path = tmp_path / "sf.yaml"
        sf_path.write_text(yaml_content, encoding="utf-8")
        sf = Sandboxfile.load(sf_path)
        cfg = sf.to_sandbox_config()
        assert cfg.name == "convert-test"
        assert cfg.memory_mb == 8192
        assert cfg.virtual_gpu == "Disable"

    def test_to_sandbox_config_rejects_hyperv(self, tmp_path):
        yaml_content = """
name: hyperv-test
base: hyperv-vm
"""
        sf_path = tmp_path / "sf.yaml"
        sf_path.write_text(yaml_content, encoding="utf-8")
        sf = Sandboxfile.load(sf_path)
        with pytest.raises(ValueError, match="windows-sandbox"):
            sf.to_sandbox_config()

    def test_render_startup_script(self, tmp_path):
        yaml_content = """
name: render-test
base: windows-sandbox
env:
  DINOFORGE_ENV: sandbox
setup:
  - run: "powershell -c 'Write-Host hi'"
  - wait_for_file: "C:/ready.flag"
    timeout_s: 30
"""
        sf_path = tmp_path / "sf.yaml"
        sf_path.write_text(yaml_content, encoding="utf-8")
        sf = Sandboxfile.load(sf_path)
        script = sf.render_startup_script("C:\\SandboxShared\\ready.flag")
        assert "DINOFORGE_ENV" in script
        assert "Write-Host hi" in script
        assert "WaitForFileStep" not in script  # not a string in output
        assert "Test-Path" in script
        assert "SANDBOX_READY" in script
        assert "ready.flag.error" in script

    def test_render_startup_script_catch_error(self, tmp_path):
        yaml_content = """
name: error-test
base: windows-sandbox
setup:
  - run: "exit 1"
"""
        sf_path = tmp_path / "sf.yaml"
        sf_path.write_text(yaml_content, encoding="utf-8")
        sf = Sandboxfile.load(sf_path)
        script = sf.render_startup_script()
        assert "catch" in script
        assert "exit 1" in script

    def test_to_hyperv_script(self, tmp_path):
        yaml_content = """
name: hyperv-gen
base: hyperv-vm
memory_mb: 8192
virtual_gpu: true
cpu_count: 4
"""
        sf_path = tmp_path / "sf.yaml"
        sf_path.write_text(yaml_content, encoding="utf-8")
        sf = Sandboxfile.load(sf_path)
        ps = sf.to_hyperv_script(
            vm_name="dino-test-vm",
            base_vhdx="C:\\HyperV\\dino-base.vhdx",
            vm_dir="C:\\HyperV\\VMs",
        )
        assert "dino-test-vm" in ps
        assert "dino-base.vhdx" in ps
        assert "New-VHD" in ps
        assert "Add-VMGpuPartitionAdapter" in ps
        assert "Start-VM" in ps


# ---------------------------------------------------------------------------
# HealthCheck tests
# ---------------------------------------------------------------------------

class TestHealthCheck:
    def test_from_dict_file(self):
        hc = HealthCheck.from_dict({"type": "file", "path": "C:\\ready.flag", "timeout_s": 45})
        assert hc.type == "file"
        assert hc.path == "C:\\ready.flag"
        assert hc.timeout_s == 45

    def test_from_dict_tcp(self):
        hc = HealthCheck.from_dict({
            "type": "tcp", "host": "172.20.0.5", "port": 8765, "timeout_s": 90
        })
        assert hc.type == "tcp"
        assert hc.host == "172.20.0.5"
        assert hc.port == 8765


# ---------------------------------------------------------------------------
# SteamlessResult tests
# ---------------------------------------------------------------------------

class TestSteamlessResult:
    def test_success(self):
        r = SteamlessResult(success=True, message="ok", stripped_files=["a.exe"], errors=[])
        assert r.success is True
        assert r.errors == []

    def test_failure(self):
        r = SteamlessResult(success=False, message="fail", stripped_files=[], errors=["exit 1"])
        assert r.success is False
        assert "exit 1" in r.errors


# ---------------------------------------------------------------------------
# SteamlessHelper tests (file-system operations)
# ---------------------------------------------------------------------------

class TestSteamlessHelper:
    def test_init_paths(self, tmp_path):
        helper = SteamlessHelper(
            source_game_dir=str(tmp_path / "source"),
            base_image_dir=str(tmp_path / "base"),
            steamless_dir=str(tmp_path / "sl"),
        )
        assert helper.source_game_dir.name == "source"
        assert helper.base_image_dir.name == "base"
        assert not helper.steamless_exe.exists()  # not downloaded yet

    def test_is_stripped_no_binary(self, tmp_path):
        helper = SteamlessHelper(
            source_game_dir=str(tmp_path / "source"),
            base_image_dir=str(tmp_path / "base"),
        )
        # No steamless binary -> False
        assert helper.is_stripped() is False

    def test_is_stripped_no_game_exe(self, tmp_path):
        helper = SteamlessHelper(
            source_game_dir=str(tmp_path / "empty"),
            base_image_dir=str(tmp_path / "base"),
        )
        # No game exe -> False
        assert helper.is_stripped() is False

    def test_to_mapped_folder(self, tmp_path):
        helper = SteamlessHelper(
            source_game_dir=str(tmp_path / "source"),
            base_image_dir=str(tmp_path / "base"),
        )
        mf = helper.to_mapped_folder("C:\\DINO")
        assert mf.host_folder == str(tmp_path / "base")
        assert mf.sandbox_folder == "C:\\DINO"
        assert mf.read_only is True

    def test_verify_stripped_exe_missing(self, tmp_path):
        helper = SteamlessHelper(
            source_game_dir=str(tmp_path / "source"),
            base_image_dir=str(tmp_path / "base"),
        )
        assert helper.verify_stripped_exe() is False

    def test_clone_to_requires_base(self, tmp_path):
        helper = SteamlessHelper(
            source_game_dir=str(tmp_path / "source"),
            base_image_dir=str(tmp_path / "nonexistent"),
        )
        with pytest.raises(FileNotFoundError):
            helper.clone_to(str(tmp_path / "target"))


# ---------------------------------------------------------------------------
# GameSandboxConfig tests
# ---------------------------------------------------------------------------

class TestGameSandboxConfig:
    def test_game_exe_auto_compute(self):
        gcfg = GameSandboxConfig(
            sandbox=SandboxConfig(name="test"),
            game_dir="C:\\DINO",
        )
        assert "Diplomacy is Not an Option.exe" in gcfg.game_exe_path
        assert "C:\\DINO" in gcfg.game_exe_path

    def test_game_exe_explicit(self):
        gcfg = GameSandboxConfig(
            sandbox=SandboxConfig(name="test"),
            game_dir="C:\\DINO",
            game_exe="C:\\Games\\DINO.exe",
        )
        assert gcfg.game_exe_path == "C:\\Games\\DINO.exe"

    def test_to_sandboxfile(self):
        sc = SandboxConfig(
            name="game-test",
            mapped_folders=[
                MappedFolder(host_folder="G:\\Games", sandbox_folder="C:\\DINO", read_only=True),
            ],
        )
        gcfg = GameSandboxConfig(sandbox=sc, game_dir="C:\\DINO")
        sf = gcfg.to_sandboxfile()
        assert sf.name == "game-test"
        assert sf.health_check["type"] == "file"
        assert sf.health_check["path"] == "C:\\SandboxShared\\game_ready.flag"
