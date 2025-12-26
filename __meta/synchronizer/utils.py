import traceback

ENVS = ["local", "dev", "staging", "prod"]


class Styler:
    def __init__(self, section):
        self.section = section

    def __enter__(self):
        print()
        print("-" * 50)
        print(f"Syncing {self.section}...")

    def __exit__(self, exc_type, exc_value, traceback):
        pass


def error(message, print_traceback=True):
    print(f"\033[91m{message}\033[0m")
    if print_traceback:
        traceback.print_exc()
