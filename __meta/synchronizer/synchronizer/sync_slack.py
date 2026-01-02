import os

from slack_sdk import WebClient
from slack_sdk.errors import SlackApiError

from synchronizer.models.contributor import Contributor
from synchronizer.models.team import Team
from synchronizer.utils import debug, error, log_operation, log_team_sync, print_section


class SlackManager:
    def __init__(
        self, contributors: dict[str, Contributor], teams: dict[str, Team]
    ) -> None:
        self.contributors = contributors
        self.teams = teams
        self.client = WebClient(token=os.getenv("SLACK_TOKEN"))

    def sync(self) -> None:
        print_section("Slack")
        for team in self.teams.values():
            self.sync_team(team)

    @log_team_sync()
    def sync_team(self, team: Team) -> None:
        # Get the desired members for the team
        desired_members = {
            self.contributors[member].slack_member_id
            for member in team.leads + team.devs
        }

        if len(team.slack_channel_ids) == 0:
            debug(f"No Slack channels to sync for {team.name}, skipping...")
            return

        # Sync each channel
        for channel_id in team.slack_channel_ids:
            with log_operation(f"sync {team.name} Slack channel: {channel_id}"):
                self.sync_channel(team, channel_id, desired_members)

    def sync_channel(
        self, team: Team, channel_id: str, desired_members: set[str]
    ) -> None:
        # Join the channel so the bot can invite users
        try:
            self.client.conversations_join(channel=channel_id)
        except SlackApiError as e:
            error(f"Error joining {team.name} Slack channel: {e.response['error']}")
            return

        # Get the current members of the channel
        try:
            response = self.client.conversations_members(channel=channel_id)
        except SlackApiError as e:
            error(
                f"Error getting members of {team.name} Slack channel: "
                f"{e.response['error']}",
            )
            return

        # Get the users to invite
        current_members = set(response["members"])
        users = list(desired_members - current_members)
        if not users:
            debug(f"No users to invite to {team.name} Slack channel.")
            return

        try:
            log_message = f"invite users to {team.name} Slack channel: {users}"
            with log_operation(log_message):
                self.client.conversations_invite(channel=channel_id, users=users)
        except SlackApiError as e:
            error(f"Error syncing {team.name} Slack channel: {e.response['error']}")
