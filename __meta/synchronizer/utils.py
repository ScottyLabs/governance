import traceback
import os

ENVS = ["local", "dev", "staging", "prod"]


def print_section(section):
    print()
    print("\033[1m" + "=" * 50 + "\033[0m")
    print("\033[1m" + f"Syncing {section}..." + "\033[0m")
    print("\033[1m" + "=" * 50 + "\033[0m")


def info(message):
    print("\033[94m" + message + "\033[0m")


def warn(message):
    print("\033[93m" + message + "\033[0m")


def error(message):
    print("\033[91m" + message + "\033[0m")
    traceback.print_exc()
