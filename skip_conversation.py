import time
import winsound
from typing import NoReturn

import pyautogui
import pygetwindow
import pynput

MX: float = 0.7160
MY: float = 0.7444
enable_sound_path: str = r"sound\enable.wav"
disable_sound_path: str = r"sound\disable.wav"


def main() -> NoReturn:
    def on_press(key) -> None:
        nonlocal on
        if key == pynput.keyboard.KeyCode(char='p'):
            window = pygetwindow.getActiveWindow()
            assert window is not None
            if window.title in ['原神', '崩坏：星穹铁道']:
                on = not on
                if on:
                    winsound.PlaySound(enable_sound_path, flags=1)
                    print('Enabled')
                else:
                    winsound.PlaySound(disable_sound_path, flags=1)
                    print('Disabled')
    on: bool = False
    listener = pynput.keyboard.Listener(on_press=on_press)
    listener.start()
    while True:
        if on:
            window = pygetwindow.getActiveWindow()
            assert window is not None
            if window.title in ['原神', '崩坏：星穹铁道']:
                x: int = round(window.left + MX * window.width)
                y: int = round(window.top + MY * window.height)
                pyautogui.moveTo(x, y)
                pyautogui.leftClick()
        time.sleep(0.1)


if __name__ == '__main__':
    main()
