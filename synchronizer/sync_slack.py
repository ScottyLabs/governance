from slack_sdk import WebClient
from slack_sdk.errors import SlackApiError
import os


class SlackManager:
    def __init__(self, contributors, teams):
        self.contributors = contributors
        self.teams = teams
        self.client = WebClient(token=os.getenv("SLACK_TOKEN"))

    def sync(self):
        print("\nSyncing Slack...")
        for team in self.teams.values():
            # Get the desired members for the team
            desired_members = set(
                [
                    self.contributors[member]["slack-member-id"]
                    for member in team["leads"] + team["devs"]
                ]
            )

            # Sync each channel
            for channel_id in team["slack-channel-ids"]:
                self.sync_channel(team, channel_id, desired_members)

    def sync_channel(self, team, channel_id, desired_members):
        print(f"Syncing {team['name']} Slack channel: {channel_id}")

        # Join the channel so the bot can invite users
        try:
            self.client.conversations_join(channel=channel_id)
        except SlackApiError as e:
            print(f"Error joining {team['name']} Slack channel: {e.response['error']}")
            return

        # Sync the members to the channel
        try:
            response = self.client.conversations_members(channel=channel_id)
            current_members = set(response["members"])
            users = list(desired_members - current_members)
            print(f"Inviting users to {team['name']} Slack channel: {users}")
            if users:
                self.client.conversations_invite(channel=channel_id, users=users)

        except SlackApiError as e:
            print(f"Error syncing {team['name']} Slack channel: {e.response['error']}")
