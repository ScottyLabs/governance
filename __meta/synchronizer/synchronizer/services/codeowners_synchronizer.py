import sys

from github.Repository import Repository

from synchronizer.clients import get_github_client
from synchronizer.logger import get_app_logger, log_operation, print_section
from synchronizer.models import Team

from .abstract_synchronizer import AbstractSynchronizer


class CodeownersSynchronizer(AbstractSynchronizer):
    REPO_NAME = "ScottyLabs/Governance"
    CODEOWNERS_FILE_PATH = ".github/CODEOWNERS"
    COMMIT_MESSAGE = "chore: auto-update CODEOWNERS"

    def __init__(self, teams: dict[str, Team]) -> None:
        self.teams = teams
        self.g = get_github_client()
        self.logger = get_app_logger()

    def sync(self) -> None:
        print_section("Codeowners")

        # Generate the new codeowners file
        new_codeowners_file = self.generate_codeowners_file()

        # Get the current codeowners file
        repo = self.g.get_repo(self.REPO_NAME)
        current_codeowners_file, sha = self.get_current_codeowners_file(repo)
        if new_codeowners_file == current_codeowners_file:
            self.logger.debug("No changes to the codeowners file. Skipping...\n")
            return

        # Update or create the codeowners file
        if sha:
            with log_operation("update the codeowners file"):
                repo.update_file(
                    self.CODEOWNERS_FILE_PATH,
                    self.COMMIT_MESSAGE,
                    new_codeowners_file,
                    sha,
                )
        else:
            with log_operation("create the codeowners file"):
                repo.create_file(
                    self.CODEOWNERS_FILE_PATH,
                    self.COMMIT_MESSAGE,
                    new_codeowners_file,
                )

    def generate_codeowners_file(self) -> str:
        file_path = "__meta/synchronizer/synchronizer/services/sync_codeowners.py"
        lines = [f"# Auto-generated CODEOWNERS file from {file_path}"]
        lines.append("")

        # Default to the tech director
        # TODO: Get the tech director from the leadership team
        lines.append("* @Yuxiang-Huang")
        lines.append("")

        # Assign the codeowners of each team as the leads of the team
        # Sort the teams to prevent changes to the codeowners file due to ordering
        for team in sorted(self.teams.values(), key=lambda x: x.slug):
            codeowners_pattern = f"teams/{team.slug}.toml"
            for lead in team.leads:
                codeowners_pattern += f" @{lead}"
            lines.append(codeowners_pattern)
            lines.append("")

        return "\n".join(lines)

    def get_current_codeowners_file(
        self, repo: Repository
    ) -> tuple[str | None, str | None]:
        """Get the current codeowners file from the repository."""
        try:
            contents = repo.get_contents(".github/CODEOWNERS")

            # Abort if the contents is a list (i.e. a directory)
            if isinstance(contents, list):
                self.logger.critical(
                    "Expected '.github/CODEOWNERS' to be a single file, "
                    "but got a directory."
                )
                sys.exit(1)

            sha = contents.sha
            current_content = contents.decoded_content.decode("utf-8")
        except Exception:
            self.logger.exception("Failed to get the current codeowners file")
            sha, current_content = None, None

        return current_content, sha
