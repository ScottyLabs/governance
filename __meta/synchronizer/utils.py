import traceback
from colorama import Fore, Style
import os

ENVS = ["local", "dev", "staging", "prod"]


def print_section(section):
    print()
    print(Style.BRIGHT + "=" * 50)
    print(Style.BRIGHT + f"Syncing {section}...")
    print(Style.BRIGHT + "=" * 50)


def info(message):
    if os.getenv("GITHUB_ACTIONS"):
        print("::notice::" + message)
    else:
        print(Fore.BLUE + message)


def warn(message):
    if os.getenv("GITHUB_ACTIONS"):
        print("::warning::" + message)
    else:
        print(Fore.YELLOW + message)


def error(message):
    if os.getenv("GITHUB_ACTIONS"):
        print("::error::" + message)
    else:
        print(Fore.RED + message)
    traceback.print_exc()
