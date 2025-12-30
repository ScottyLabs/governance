import traceback
from colorama import Fore, Style

ENVS = ["local", "dev", "staging", "prod"]


def print_section(section):
    print()
    print(Style.BRIGHT + "=" * 50)
    print(Style.BRIGHT + f"Syncing {section}...")
    print(Style.BRIGHT + "=" * 50)


def info(message):
    print(Fore.BLUE + message)


def warn(message):
    print(Fore.YELLOW + message)


def error(message):
    print(Fore.RED + message)
    traceback.print_exc()
