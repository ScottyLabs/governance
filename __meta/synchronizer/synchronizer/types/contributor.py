# Automatically generated file from a JSON schema


from typing import Required, TypedDict

Contributor = TypedDict(
    "Contributor",
    {
        # | Required property
        "full-name": Required[str],
        # | Required property
        "github-username": Required[str],
        # | pattern: ^U[A-Z0-9]+$
        # |
        # | Required property
        "slack-member-id": Required[str],
        # | pattern: ^[a-z0-9]+$
        "andrew-id": str,
    },
    total=False,
)
