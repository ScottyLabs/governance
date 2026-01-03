import os
import sys

from slack_sdk import WebClient
from slack_sdk.errors import SlackApiError

from synchronizer.logger import (
    get_app_logger,
    log_operation,
    log_team_sync,
    print_section,
)
from synchronizer.models import Contributor, Team


class SlackManager:
    def __init__(
        self, contributors: dict[str, Contributor], teams: dict[str, Team]
    ) -> None:
        self.logger = get_app_logger()
        self.contributors = contributors
        self.teams = teams

        slack_token = os.getenv("SLACK_TOKEN")
        if not slack_token:
            self.logger.critical("SLACK_TOKEN is not set")
            sys.exit(1)

        self.client = WebClient(token=slack_token)

    def sync(self) -> None:
        print_section("Slack")
        for team in self.teams.values():
            self.sync_team(team)

    @log_team_sync()
    def sync_team(self, team: Team) -> None:
        # Get the desired members' Slack IDs for the team
        desired_members = set()
        for member in team.leads + team.devs:
            contributor = self.contributors[member]
            if contributor.slack_member_id is None:
                self.logger.warning(
                    "Contributor %s does not have a Slack member ID.\n",
                    member,
                )
                continue

            desired_members.add(contributor.slack_member_id)

        if len(team.slack_channel_ids) == 0:
            self.logger.debug(
                "No Slack channels to sync for %s, skipping...",
                team.name,
            )
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
            self.logger.critical(
                "Error joining %s Slack channel: %s",
                team.name,
                e.response["error"],
            )
            return
        except Exception:
            self.logger.exception(
                "Error joining %s Slack channel",
                team.name,
            )
            return

        # Get the current members of the channel
        try:
            response = self.client.conversations_members(channel=channel_id)
        except SlackApiError as e:
            self.logger.exception(
                "Error getting members of %s Slack channel: %s",
                team.name,
                e.response["error"],
            )
            return
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

        try:
            log_message = f"invite users to {team.name} Slack channel: {users}"
            with log_operation(log_message):
                self.client.conversations_invite(channel=channel_id, users=users)
        except SlackApiError as e:
            self.logger.exception(
                "Error syncing %s Slack channel: %s",
                team.name,
                e.response["error"],
            )
        except Exception:
            self.logger.exception(
                "Error syncing %s Slack channel",
                team.name,
            )
