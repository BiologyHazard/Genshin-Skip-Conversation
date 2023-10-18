import logging
import time
import winsound
from pathlib import Path
from typing import Never

import pyautogui
import pygetwindow
import pynput

logging.basicConfig(level=logging.DEBUG,
                    format='{asctime}.{msecs:03.0f} [{levelname}] {module}:{funcName}:{lineno} | {message}',
                    datefmt='%Y-%m-%d %H:%M:%S',
                    style='{')

enable_sound_path: Path = Path(__file__).parent / 'sound/enable.wav'
disable_sound_path: Path = Path(__file__).parent / 'sound/disable.wav'

skip_conversation_clicks: tuple[float, float] = (0.7160, 0.7444)
skip_conversation_supported_titles: list[str] = ['原神', '崩坏：星穹铁道', 'Genshin Impact']
skip_conversation_key_code = pynput.keyboard.KeyCode(char='p')
skip_conversation_click_interval: float = 0.1  # 单位秒


enhance_aritifact_clicks: list[tuple[float, float]] = [
    (241/3268, 433/1892),
    (2973/3268, 1367/1892),
    (2953/3268, 1787/1892),
    (244/3268, 302/1892),
]
enhance_aritifact_supported_titles: list[str] = ['原神', 'Genshin Impact']
enhance_aritifact_key_code = pynput.keyboard.KeyCode(char='[')


def main() -> Never:
    def on_press(key) -> None:
        nonlocal on
        if key == skip_conversation_key_code:
            window = pygetwindow.getActiveWindow()
            if window is not None and window.title in skip_conversation_supported_titles:
                on = not on
                if on:
                    winsound.PlaySound(str(enable_sound_path), flags=1)
                    logging.info('Skip Conversation Enabled')
                else:
                    winsound.PlaySound(str(disable_sound_path), flags=1)
                    logging.info('Skip Conversation Disabled')
        if key == enhance_aritifact_key_code:
            window = pygetwindow.getActiveWindow()
            if window is not None and window.title in enhance_aritifact_supported_titles:
                logging.info('Enhance Aritifact')
                for mx, my in enhance_aritifact_clicks:
                    x: int = round(window.left + mx * window.width)
                    y: int = round(window.top + my * window.height)
                    # pyautogui.moveTo(x, y)
                    pyautogui.leftClick(x, y)
                    # time.sleep(0.1)

    on: bool = False
    listener = pynput.keyboard.Listener(on_press=on_press)
    listener.start()
    while True:
        if on:
            window = pygetwindow.getActiveWindow()
            if window is not None and window.title in skip_conversation_supported_titles:
                mx, my = skip_conversation_clicks
                x: int = round(window.left + mx * window.width)
                y: int = round(window.top + my * window.height)
                # pyautogui.moveTo(x, y)
                pyautogui.leftClick(x, y)
        time.sleep(skip_conversation_click_interval)


if __name__ == '__main__':
    main()
