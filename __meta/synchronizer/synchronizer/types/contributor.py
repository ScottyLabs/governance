from typing import TypedDict


class Contributor(TypedDict):
    full_name: str
    github_username: str
    slack_member_id: str
    andrew_id: str | None
