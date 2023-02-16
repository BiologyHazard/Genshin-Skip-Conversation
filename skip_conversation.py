import time
from typing import NoReturn

import pyautogui
import pygetwindow
from pynput import keyboard
from winsound import PlaySound


MX: float = 0.7160
MY: float = 0.7444
enable_sound_path: str = r"sound\enable.wav"
disable_sound_path: str = r"sound\disable.wav"


def main() -> NoReturn:
    def on_press(key) -> None:
        nonlocal flag
        if key == keyboard.KeyCode(char='p'):
            windows: list[pygetwindow.Window] = pygetwindow.getWindowsWithTitle(
                '原神')
            if windows:
                window = windows[0]
                if window.isActive:
                    flag = not flag
                    if flag:
                        PlaySound(enable_sound_path, flags=1)
                        print('Enabled')
                    else:
                        PlaySound(disable_sound_path, flags=1)
                        print('Disabled')
    flag: bool = False
    listener = keyboard.Listener(on_press=on_press)
    listener.start()
    while True:
        if flag:
            windows: list[pygetwindow.Window] = pygetwindow.getWindowsWithTitle(
                '原神')
            if windows:
                window = windows[0]
                if window.isActive:
                    x: int = round(window.left + MX * window.width)
                    y: int = round(window.top + MY * window.height)
                    pyautogui.moveTo(x, y)
                    pyautogui.leftClick()
        time.sleep(0.1)


if __name__ == '__main__':
    main()
