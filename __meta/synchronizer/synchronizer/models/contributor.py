from typing import TypedDict

Contributor = TypedDict(
    "Contributor",
    {
        "full-name": str,
        "github-username": str,
        "slack-member-id": str,
        "andrew-id": str | None,
    },
)
