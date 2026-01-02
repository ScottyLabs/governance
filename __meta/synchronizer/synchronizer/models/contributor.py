from pydantic import BaseModel, Field


class Contributor(BaseModel):
    full_name: str = Field(alias="full-name")
    github_username: str = Field(alias="github-username")
    slack_member_id: str = Field(alias="slack-member-id")
    andrew_id: str | None = Field(alias="andrew-id", default=None)
