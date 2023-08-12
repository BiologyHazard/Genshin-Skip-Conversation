import time
import winsound
from pathlib import Path
from typing import NoReturn

import pyautogui
import pygetwindow
import pynput

MX: float = 0.7160
MY: float = 0.7444
enable_sound_path: Path = Path(__file__).parent / 'sound/enable.wav'
disable_sound_path: Path = Path(__file__).parent / 'sound/disable.wav'
supported_titles: list[str] = ['原神', '崩坏：星穹铁道', 'Genshin Impact']
key_code = pynput.keyboard.KeyCode(char='p')
click_interval: float = 0.1  # 单位秒


def main() -> NoReturn:
    def on_press(key) -> None:
        nonlocal on
        if key == key_code:
            window = pygetwindow.getActiveWindow()
            assert window is not None
            if window.title in supported_titles:
                on = not on
                if on:
                    winsound.PlaySound(str(enable_sound_path), flags=1)
                    print('Enabled')
                else:
                    winsound.PlaySound(str(disable_sound_path), flags=1)
                    print('Disabled')
    on: bool = False
    listener = pynput.keyboard.Listener(on_press=on_press)
    listener.start()
    while True:
        if on:
            window = pygetwindow.getActiveWindow()
            assert window is not None
            if window.title in supported_titles:
                x: int = round(window.left + MX * window.width)
                y: int = round(window.top + MY * window.height)
                pyautogui.moveTo(x, y)
                pyautogui.leftClick()
        time.sleep(click_interval)


if __name__ == '__main__':
    main()
