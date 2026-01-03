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
    _logger: ClassVar[AppLogger | None] = None

    def __new__(cls) -> "AppLoggerSingleton":
        if cls._instance is None:
            with cls._lock:  # Ensure only one thread initializes it
                if cls._instance is None:
                    cls._instance = super().__new__(cls)
                    cls._init_logger()
        return cls._instance

    @classmethod
    def _init_logger(cls) -> None:
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

        # Set the logger
        cls._logger = cast("AppLogger", app_logger)

    @classmethod
    def get_logger(cls) -> AppLogger:
        """Return the singleton logger instance."""
        # Lazily instantiate the singleton
        if cls._instance is None:
            cls()

        # Raise an error if the logger is still not initialized
        if cls._logger is None:
            msg = "Logger not initialized"
            raise RuntimeError(msg)

        # Return the logger
        return cls._logger
