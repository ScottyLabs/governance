from abc import ABC, abstractmethod

from synchronizer.logger import get_app_logger
from synchronizer.models import Contributor, Repo, Team


class AbstractSynchronizer(ABC):
    @abstractmethod
    def __init__(
        self,
        contributors: dict[str, Contributor],
        teams: dict[str, Team],
        *,
        repos: dict[str, Repo] | None = None,
    ) -> None:
        """
        Initialize the AbstractSynchronizer.

        Sets the contributors, teams, and registered repos (if any). Repos are the
        source of truth for repository metadata and URLs (GitHub, Codeberg, etc.).
        """
        self.contributors = contributors
        self.teams = teams
        self.repos = repos or {}
        self.logger = get_app_logger()

    @abstractmethod
    def sync(self) -> None:
        msg = "Subclasses must implement this method"
        raise NotImplementedError(msg)
