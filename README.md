# bare-cua

[![AI Slop Inside](https://sladge.net/badge.svg)](https://sladge.net)

Minimal headless browser automation and device control framework enabling programmatic UI interaction without external service dependencies.

## Overview

bare-cua provides lightweight automation primitives for interacting with web UIs, mobile interfaces, and desktop applications. It abstracts browser/OS complexity, enabling agents to automate interactions, capture screenshots, perform OCR, and control devices in a unified manner. Designed for autonomous system integration without heavy external service requirements.

## Technology Stack

- **Languages**: Python, Go, Zig (polyglot framework)
- **Core**: Playwright, Chromium, native OS APIs, FFmpeg
- **Key Dependencies**: `playwright`, `asyncio`, `pillow`, `pyautogui`, `easyocr`
- **Architecture**: Plugin-based executor pattern
- **Deployment**: Native binaries, Docker, cloud-native

## Key Features

- Headless browser automation (Chrome, Firefox, Safari)
- Screenshot and OCR capabilities
- Form interaction and text input
- Keyboard and mouse simulation
- Native window management
- Device screen recording
- Cross-platform support (macOS, Linux, Windows)
- Session persistence and recovery
- Concurrent automation with resource pooling

## Quick Start

```bash
# Clone repository
git clone https://github.com/KooshaPari/bare-cua.git
cd bare-cua

# Review governance
cat CLAUDE.md

# Install dependencies
python -m pip install -e ".[dev]"

# Run tests
pytest tests/

# Start automation server
python -m bare_cua.server --port 9000

# Send command
curl -X POST http://localhost:9000/interact \
  -H "Content-Type: application/json" \
  -d '{"action":"navigate","url":"https://example.com"}'
```

## Project Structure

```
bare-cua/
├── src/
│   ├── browser/
│   │   ├── chromium.py        # Chromium driver
│   │   ├── firefox.py         # Firefox driver
│   │   └── manager.py         # Browser lifecycle
│   ├── input/
│   │   ├── keyboard.py        # Keyboard events
│   │   ├── mouse.py           # Mouse/pointer events
│   │   └── touch.py           # Touch/gesture support
│   ├── capture/
│   │   ├── screenshot.py      # Screen capture
│   │   ├── video.py           # Video recording
│   │   └── ocr.py             # Text recognition
│   ├── commands/
│   │   ├── navigate.py        # Navigation
│   │   ├── interact.py        # DOM interaction
│   │   ├── extract.py         # Data extraction
│   │   └── execute.py         # Script execution
│   ├── server.py              # HTTP API
│   └── types.py               # Type definitions
├── tests/
│   ├── unit/                  # Unit tests
│   └── integration/           # Browser tests
├── docs/
│   ├── ARCHITECTURE.md        # Design documentation
│   ├── COMMANDS.md            # Command reference
│   └── EXAMPLES.md            # Usage examples
└── pyproject.toml             # Python packaging
```

## Related Phenotype Projects

- **[KDesktopVirt](../KDesktopVirt)** — Desktop virtualization
- **[KVirtualStage](../KVirtualStage)** — Virtual display/sandbox
- **[agentkit](../agentkit)** — Agent framework integration

## Governance & Documentation

- **CLAUDE.md** — Development guidelines and patterns
- **docs/ARCHITECTURE.md** — Design and architecture

## License

MIT — see [LICENSE](./LICENSE).

---

**Status**: Active development  
**Maintained by**: Phenotype Org  
**Last Updated**: 2026-04-24
