from abc import ABC, abstractmethod

from synchronizer.logger import get_app_logger
from synchronizer.models import Contributor, Team


class AbstractSynchronizer(ABC):
    @abstractmethod
    def __init__(
        self, contributors: dict[str, Contributor], teams: dict[str, Team]
    ) -> None:
        """
        Initialize the AbstractSynchronizer.

        Sets the contributors and teams and creates a logger.
        """
        self.contributors = contributors
        self.teams = teams
        self.logger = get_app_logger()

    @abstractmethod
    def sync(self) -> None:
        msg = "Subclasses must implement this method"
        raise NotImplementedError(msg)
