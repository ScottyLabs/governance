import os
import sys
from typing import Optional

from github import Auth, Github

from synchronizer.logger import AppLoggerSingleton


class GithubClient:
    """Singleton wrapper around GithubAdmin for reuse across the app."""

    _instance: Optional["GithubClient"] = None
    _g: Github | None = None

    def __new__(cls) -> "GithubClient":
        if cls._instance is None:
            cls._instance = super().__new__(cls)
            cls._instance.init_g()
        return cls._instance

    def init_g(self) -> None:
        """Initialize the Github admin client once."""
        self.logger = AppLoggerSingleton().logger

        github_token = os.getenv("SYNC_GITHUB_TOKEN")
        if not github_token:
            msg = "SYNC_GITHUB_TOKEN is not set"
            self.logger.critical(msg)
            sys.exit(1)

        self._g = Github(auth=Auth.Token(github_token))

    @property
    def g(self) -> Github:
        self.logger = AppLoggerSingleton().logger

        if self._g is None:
            msg = "Github client not initialized"
            self.logger.critical(msg)
            sys.exit(1)

        return self._g
