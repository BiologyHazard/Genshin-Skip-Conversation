import time

import pyautogui
import pygetwindow
from pynput import keyboard


MX = 0.7160
MY = 0.7444


def on_press(key):
    global flag
    if key == keyboard.KeyCode(char='p'):
        flag = not flag
        print(('PAUSED', 'RESUMED')[int(flag)])


def main():
    global flag
    flag = False
    listener = keyboard.Listener(on_press=on_press)
    listener.start()
    while True:
        if flag:
            windows: list[pygetwindow.Window] = pygetwindow.getWindowsWithTitle(
                '原神')
            if windows:
                window = windows[0]
                if window.isActive:
                    x = round(window.left + MX * window.width)
                    y = round(window.top + MY * window.height)
                    pyautogui.moveTo(x, y)
                    pyautogui.leftClick()
        time.sleep(0.2)


if __name__ == '__main__':
    main()
