from abc import ABC, abstractmethod


class AbstractSynchronizer(ABC):
    @abstractmethod
    def sync(self) -> None:
        msg = "Subclasses must implement this method"
        raise NotImplementedError(msg)
