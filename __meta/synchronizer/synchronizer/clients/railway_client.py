import os
from typing import Any, ClassVar

import requests


class RailwayClient:
    """Python client wrapper of the Railway GraphQL API for synchronization."""

    API_URL = "https://backboard.railway.com/graphql/v2"

    HEADERS: ClassVar[dict[str, str]] = {
        "Authorization": f"Bearer {os.getenv('RAILWAY_TOKEN')}",
        "Content-Type": "application/json",
    }

    TIMEOUT = 10  # timeout in seconds

    ADMIN_ROLE = "MEMBER"
    DEV_ROLE = "VIEWER"

    @classmethod
    def get_workspace_members(cls, workspace_id: str) -> dict:
        query = """
        query workspaceMembers($workspaceId: String!) {
            workspace(workspaceId: $workspaceId) {
                members {
                    email
                    role
                }
            }
        }
        """
        variables = {"workspaceId": workspace_id}
        return cls._query(query, variables)

    @classmethod
    def get_projects(cls) -> dict:
        query = """
        query {
            projects {
                edges {
                    node {
                        id
                        name
                        members {
                            id
                            email
                            role
                        }
                    }
                }
            }
        }
        """
        return cls._query(query, {})

    @classmethod
    def get_project_invitations(cls, project_id: str) -> dict:
        query = """
        query ProjectInvitations($id: String!) {
        projectInvitations(id: $id) {
            email
        }
        }
        """
        variables = {"id": project_id}
        return cls._query(query, variables)

    @classmethod
    def get_project_invite_link(cls, project_id: str, role: str) -> str:
        query = """
        query projectInviteCode($projectId: String!, $role: ProjectRole!) {
            projectInviteCode(projectId: $projectId, role: $role) {
                code
            }
        }
        """
        variables = {
            "projectId": project_id,
            "role": role,
        }
        data = cls._query(query, variables)
        code = data.get("projectInviteCode", {}).get("code")
        return f"https://railway.com/invite/{code}"

    @classmethod
    def invite_project_member(cls, project_id: str, email: str, link: str) -> None:
        mutation = r"""
        mutation AddProjectMember(
            $projectId: String!
            $email: String!
            $link: String!
        ) {
            projectInviteUser(
                id: $projectId
                input: { email: $email, link: $link }
            )
        }
        """

        variables = {
            "projectId": project_id,
            "email": email,
            "link": link,
        }

        cls._mutation(mutation, variables)

    @classmethod
    def remove_project_member(cls, project_id: str, user_id: str) -> None:
        mutation = """
        mutation projectMemberRemove($input: ProjectMemberRemoveInput!) {
            projectMemberRemove(input: $input) {
            }
        }
        """
        variables = {
            "input": {
                "projectId": project_id,
                "userId": user_id,
            },
        }
        cls._mutation(mutation, variables)

    @classmethod
    def _query(cls, query: str, variables: dict[str, Any]) -> dict:
        response = requests.post(
            cls.API_URL,
            json={"query": query, "variables": variables},
            headers=cls.HEADERS,
            timeout=cls.TIMEOUT,
        )
        return response.json()

    @classmethod
    def _mutation(cls, mutation: str, variables: dict[str, Any]) -> dict:
        response = requests.post(
            cls.API_URL,
            json={"mutation": mutation, "variables": variables},
            headers=cls.HEADERS,
            timeout=cls.TIMEOUT,
        )
        return response.json()
