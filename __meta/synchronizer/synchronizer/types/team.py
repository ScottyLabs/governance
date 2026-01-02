from typing import Literal, TypedDict


class Team(TypedDict):
    slug: str
    name: str
    website_slug: str | None
    leads: list[str]
    devs: list[str] | None
    applicants: list[str] | None
    ext_admins: list[str] | None
    repos: list[str]
    slack_channel_ids: list[str] | None
    remove_unlisted: bool | None
    create_oidc_clients: bool | None
    secrets_population_layout: Literal["single", "multi", "none"] | None
