import logging
import threading
from typing import ClassVar, cast

from colorama import init

from .components import AppLogger, ColorFormatter, LogStatusFilter
from .constants import PRINT_LEVEL, SUCCESS_LEVEL


class AppLoggerSingleton:
    """Singleton wrapper around the logger."""

    _instance: ClassVar["AppLoggerSingleton | None"] = None
    _lock: ClassVar[threading.Lock] = threading.Lock()
    logger: AppLogger

    def __new__(cls) -> "AppLoggerSingleton":
        if cls._instance is None:
            with cls._lock:  # Ensure only one thread initializes it
                if cls._instance is None:
                    instance = super().__new__(cls)
                    instance.logger = cls._init_logger()
                    cls._instance = instance
        return cls._instance

    @staticmethod
    def _init_logger() -> AppLogger:
        # Register the success and print log levels
        logging.addLevelName(SUCCESS_LEVEL, "SUCCESS")
        logging.addLevelName(PRINT_LEVEL, "PRINT")

        # Set the custom logger class
        logging.setLoggerClass(AppLogger)

        # Colorama: ensures reset after each print and force keep ANSI for colors
        init(autoreset=True, strip=False)

        # Color handler for the logger
        handler = logging.StreamHandler()
        handler.setFormatter(ColorFormatter("[%(levelname)s] %(message)s"))

        # Create the logger
        app_logger = logging.getLogger(__name__)
        if app_logger.hasHandlers():
            app_logger.handlers.clear()
        app_logger.addHandler(handler)
        app_logger.setLevel(PRINT_LEVEL)

        # Add the filter to the logger
        app_logger.addFilter(LogStatusFilter())

        # Return the logger
        return cast("AppLogger", app_logger)
