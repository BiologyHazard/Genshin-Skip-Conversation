import time
import winsound
from typing import NoReturn

import pyautogui
import pygetwindow
import pynput

clicks: list[tuple[float, float]] = [
    (241/3268, 433/1892),
    (2973/3268, 1367/1892),
    (2953/3268, 1787/1892),
    (244/3268, 302/1892),
]
supported_titles: list[str] = ['原神', 'Genshin Impact']
key_code = pynput.keyboard.KeyCode(char='[')


def main() -> NoReturn:
    def on_press(key) -> None:
        if key == key_code:
            window = pygetwindow.getActiveWindow()
            assert window is not None
            if window.title in supported_titles:
                for mx, my in clicks:
                    x = round(window.left + mx * window.width)
                    y: int = round(window.top + my * window.height)
                    pyautogui.moveTo(x, y)
                    pyautogui.leftClick()
                    # time.sleep(0.1)
    listener = pynput.keyboard.Listener(on_press=on_press)
    listener.start()
    while True:
        time.sleep(0.01)


if __name__ == '__main__':
    main()
