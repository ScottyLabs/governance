from typing import Literal

from pydantic import BaseModel, Field


class Team(BaseModel):
    slug: str
    name: str
    website_slug: str | None = Field(alias="website-slug", default=None)
    leads: list[str]
    devs: list[str]
    applicants: list[str] | None = None
    ext_admins: list[str] | None = Field(alias="ext-admins", default=None)
    repos: list[str]
    slack_channel_ids: list[str] = Field(alias="slack-channel-ids")
    remove_unlisted: bool = Field(alias="remove-unlisted", default=True)
    sync_github: bool = Field(alias="sync-github", default=True)
    create_oidc_clients: bool = Field(alias="create-oidc-clients", default=True)
    secrets_population_layout: Literal["single", "multi", "none"] = Field(
        alias="secrets-population-layout", default="multi"
    )
