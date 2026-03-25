"""bare_cua.agent - screenshot-action loop agent using the Anthropic SDK.

Drives a Computer instance using Claude claude-sonnet-4-5 (or configurable model) to
complete computer-use tasks autonomously. Uses a screenshot -> LLM -> action
loop until the model signals completion or max_steps is reached.
"""

from __future__ import annotations

import base64
import json
import re
from typing import Any

import anthropic

from .computer import Computer

__all__ = ["ComputerAgent"]

# Actions the LLM can emit (subset of full computer-use spec).
ACTION_SCHEMA = """
You control a computer. At each step you will receive a screenshot and must
output a JSON action block wrapped in <action>...</action> tags.

Available actions:
  {"action": "screenshot"}
  {"action": "left_click", "x": <int>, "y": <int>}
  {"action": "right_click", "x": <int>, "y": <int>}
  {"action": "double_click", "x": <int>, "y": <int>}
  {"action": "type", "text": "<string>"}
  {"action": "key", "key": "<key_name>"}
  {"action": "scroll", "x": <int>, "y": <int>, "direction": "up|down|left|right", "amount": <int>}
  {"action": "move", "x": <int>, "y": <int>}
  {"action": "done", "result": "<summary of what was accomplished>"}

Rules:
- Always output exactly one <action>...</action> block.
- Use "done" when the task is complete or cannot be completed.
- Coordinates are absolute screen pixels.
"""


class ComputerAgent:
    """Agent that drives a Computer via a screenshot-action loop.

    Parameters
    ----------
    computer:
        A ``Computer`` instance (must be started via ``async with`` before calling ``run``).
    model:
        Anthropic model to use. Defaults to claude-claude-sonnet-4-5.
    system_prompt:
        Optional additional system instructions prepended to the action schema.
    max_steps:
        Maximum number of action steps before giving up.
    api_key:
        Anthropic API key. Defaults to ``ANTHROPIC_API_KEY`` env var.
    """

    def __init__(
        self,
        computer: Computer,
        model: str = "claude-sonnet-4-5",
        system_prompt: str = "",
        max_steps: int = 50,
        api_key: str | None = None,
    ) -> None:
        self._computer = computer
        self._model = model
        self._system = (system_prompt + "\n\n" if system_prompt else "") + ACTION_SCHEMA
        self._max_steps = max_steps
        self._client = anthropic.Anthropic(api_key=api_key) if api_key else anthropic.Anthropic()

    async def run(self, task: str) -> str:
        """Execute a natural-language task. Returns a summary string."""
        messages: list[dict[str, Any]] = []
        step = 0

        # Seed with task + initial screenshot.
        png = await self._computer.screenshot()
        messages.append({
            "role": "user",
            "content": [
                {
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": "image/png",
                        "data": base64.b64encode(png).decode(),
                    },
                },
                {"type": "text", "text": f"Task: {task}\n\nHere is the current screen. What is your first action?"},
            ],
        })

        while step < self._max_steps:
            step += 1

            # Ask the model.
            response = self._client.messages.create(
                model=self._model,
                max_tokens=1024,
                system=self._system,
                messages=messages,
            )
            reply_text = response.content[0].text if response.content else ""

            # Append assistant turn.
            messages.append({"role": "assistant", "content": reply_text})

            # Parse action.
            action = _parse_action(reply_text)
            if action is None:
                # Model returned no action block - treat as done.
                return f"Agent stopped (no action in step {step}): {reply_text}"

            action_type = action.get("action", "")

            if action_type == "done":
                return action.get("result", "Task completed.")

            # Execute the action.
            result_text = await self._execute_action(action)

            # Capture new screenshot and append as next user turn.
            png = await self._computer.screenshot()
            messages.append({
                "role": "user",
                "content": [
                    {
                        "type": "image",
                        "source": {
                            "type": "base64",
                            "media_type": "image/png",
                            "data": base64.b64encode(png).decode(),
                        },
                    },
                    {
                        "type": "text",
                        "text": f"Action executed: {result_text}\n\nStep {step}/{self._max_steps}. What is your next action?",
                    },
                ],
            })

        return f"Agent reached max_steps={self._max_steps} without completing the task."

    async def _execute_action(self, action: dict[str, Any]) -> str:
        """Execute a parsed action object on the computer. Returns a description."""
        action_type = action.get("action", "")

        if action_type == "screenshot":
            await self._computer.screenshot()
            return "screenshot captured"

        elif action_type == "left_click":
            x, y = int(action["x"]), int(action["y"])
            await self._computer.left_click(x, y)
            return f"left_click({x}, {y})"

        elif action_type == "right_click":
            x, y = int(action["x"]), int(action["y"])
            await self._computer.right_click(x, y)
            return f"right_click({x}, {y})"

        elif action_type == "double_click":
            x, y = int(action["x"]), int(action["y"])
            await self._computer.double_click(x, y)
            return f"double_click({x}, {y})"

        elif action_type == "type":
            text = str(action.get("text", ""))
            await self._computer.type_text(text)
            return f"type({text!r})"

        elif action_type == "key":
            key = str(action.get("key", ""))
            await self._computer.press_key(key)
            return f"key({key!r})"

        elif action_type == "scroll":
            x = int(action.get("x", 0))
            y = int(action.get("y", 0))
            direction = str(action.get("direction", "down"))
            amount = int(action.get("amount", 3))
            await self._computer.scroll(x, y, direction=direction, amount=amount)
            return f"scroll({x}, {y}, {direction}, {amount})"

        elif action_type == "move":
            x, y = int(action["x"]), int(action["y"])
            await self._computer.move_mouse(x, y)
            return f"move({x}, {y})"

        else:
            return f"unknown action: {action_type}"


def _parse_action(text: str) -> dict[str, Any] | None:
    """Extract and parse the first <action>...</action> JSON block from text."""
    match = re.search(r"<action>(.*?)</action>", text, re.DOTALL)
    if not match:
        return None
    raw = match.group(1).strip()
    try:
        obj = json.loads(raw)
        if isinstance(obj, dict):
            return obj
    except json.JSONDecodeError:
        pass
    return None
