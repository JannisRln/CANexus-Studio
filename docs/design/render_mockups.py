#!/usr/bin/env python3
"""Render proposed CANexus UX mockups from the standalone HTML prototype.

Requires: pip install playwright && playwright install chromium
No CAN hardware or application runtime is needed.
"""
from pathlib import Path

from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parent
VIEWS = (
    ("analyzer", "mock-01-analyzer.png"),
    ("measurement", "mock-02-measurement.png"),
    ("diagnostics", "mock-03-diagnostics.png"),
    ("design", "mock-04-design-system.png"),
)


def main() -> None:
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(headless=True)
        page = browser.new_page(viewport={"width": 1600, "height": 960}, device_scale_factor=1)
        errors = []
        page.on("pageerror", lambda error: errors.append(str(error)))
        page.goto((ROOT / "PROTOTYPE.html").as_uri(), wait_until="load")
        for view, filename in VIEWS:
            page.evaluate("(view) => nav(view)", view)
            page.screenshot(path=str(ROOT / filename))
            print(f"Rendered {filename}")
        browser.close()
        if errors:
            raise RuntimeError("Prototype JavaScript errors:\n" + "\n".join(errors))


if __name__ == "__main__":
    main()
