from pydantic import BaseModel


class Repo(BaseModel):
    """Registered repository: the source of truth for repo metadata and URL (GitHub, Codeberg, etc.)."""

    slug: str
    name: str
    description: str | None = None
    url: str
