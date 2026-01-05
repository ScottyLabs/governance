import os

from slack_sdk import WebClient
from slack_sdk.errors import SlackApiError

from synchronizer.logger import (
    log_operation,
    log_team_sync,
    print_section,
)
from synchronizer.models import Contributor, Team

from .abstract_synchronizer import AbstractSynchronizer


class SlackSynchronizer(AbstractSynchronizer):
    def __init__(
        self, contributors: dict[str, Contributor], teams: dict[str, Team]
    ) -> None:
        super().__init__(contributors, teams)

        # Initialize the Slack client
        slack_token = os.getenv("SLACK_TOKEN")
        if not slack_token:
            msg = "SLACK_TOKEN is not set"
            self.logger.critical(msg)
            raise RuntimeError(msg)

        self.client = WebClient(token=slack_token)

    def sync(self) -> None:
        print_section("Slack")
        for team in self.teams.values():
            self.sync_team(team)

    @log_team_sync()
    def sync_team(self, team: Team) -> None:
        # Skip if the team does not have any Slack channels to sync
        if len(team.slack_channel_ids) == 0:
            self.logger.debug(
                "No Slack channels to sync for %s, skipping...\n",
                team.name,
            )
            return

        # Get the desired members' Slack IDs for the team
        desired_members = set()
        for contributor_id in team.contributors:
            contributor = self.contributors[contributor_id]
            if contributor.slack_member_id is None:
                self.logger.warning(
                    "Contributor %s does not have a Slack member ID.\n",
                    contributor.full_name,
                )
                continue

            desired_members.add(contributor.slack_member_id)

        # Sync each channel
        for channel_id in team.slack_channel_ids:
            self.logger.print(f"Syncing {team.name} Slack channel: {channel_id}...\n")
            self.sync_channel(team, channel_id, desired_members)
            self.logger.print("")

    def sync_channel(
        self, team: Team, channel_id: str, desired_members: set[str]
    ) -> None:
        # Get the channel info
        try:
            info = self.client.conversations_info(channel=channel_id)
        except SlackApiError as e:
            # Trying to get info a private channel results in a channel_not_found error
            if e.response["error"] == "channel_not_found":
                msg = "The Slack Governance App need to be added to channel "
                msg += f"{channel_id} in order to invite users to the channel."
                self.logger.exception(msg)
                return
        except Exception:
            self.logger.exception(
                "Error getting info of %s Slack channel",
                team.name,
            )
            return

        # Add the Governance App to the channel if it is not already a member
        # so it can invite users to the channel
        if not info["channel"]["is_member"]:
            with log_operation(f"join Slack channel: {channel_id}"):
                self.client.conversations_join(channel=channel_id)

        # Get the current members of the channel
        try:
            response = self.client.conversations_members(channel=channel_id)
        except Exception:
            self.logger.exception(
                "Error getting members of %s Slack channel",
                team.name,
            )
            return

        # Get the users to invite
        current_members = set(response["members"])
        users = list(desired_members - current_members)
        if not users:
            self.logger.debug(
                "No users to invite to %s Slack channel.",
                team.name,
            )
            return

        # Invite the users to the channel
        log_message = f"invite users to {team.name} Slack channel: {users}"
        with log_operation(log_message):
            self.client.conversations_invite(channel=channel_id, users=users)
