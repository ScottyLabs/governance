from typing import Literal, TypedDict

Team = TypedDict(
    "Team",
    {
        "slug": str,
        "name": str,
        "website-slug": str | None,
        "leads": list[str],
        "devs": list[str],
        "applicants": list[str] | None,
        "ext-admins": list[str] | None,
        "repos": list[str],
        "slack_channel_ids": list[str] | None,
        "remove-unlisted": bool | None,
        "create-oidc-clients": bool | None,
        "secrets-population-layout": Literal["single", "multi", "none"] | None,
    },
)
