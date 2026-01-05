from github.Repository import Repository

from synchronizer.clients import get_github_client
from synchronizer.logger import log_operation, print_section
from synchronizer.models import Contributor, Team

from .abstract_synchronizer import AbstractSynchronizer


class CodeownersSynchronizer(AbstractSynchronizer):
    REPO_NAME = "ScottyLabs/Governance"
    CODEOWNERS_FILE_PATH = ".github/CODEOWNERS"
    COMMIT_MESSAGE = "chore: auto-update CODEOWNERS"

    def __init__(
        self, contributors: dict[str, Contributor], teams: dict[str, Team]
    ) -> None:
        super().__init__(contributors, teams)
        self.g = get_github_client()

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
        """Generate the CODEOWNERS file."""
        file_path = "__meta/synchronizer/synchronizer/services/sync_codeowners.py"
        lines = [f"# Auto-generated CODEOWNERS file from {file_path}"]
        lines.append("")

        # Default to the maintainers of the governance team
        if "governance" not in self.teams:
            msg = "Governance team not found."
            self.logger.critical(msg)
            raise ValueError(msg)

        governance_team = self.teams["governance"]
        lines.append("# Default owners are the maintainers of the governance team")
        codeowners_pattern = "*"
        for maintainer in sorted(governance_team.maintainers):
            codeowners_pattern += f" @{maintainer}"
        lines.append(codeowners_pattern)
        lines.append("")

        # Default owner of teams directory to the tech director,
        # which is the first maintainer of the leadership team.
        if "leadership" not in self.teams:
            msg = "Leadership team not found."
            self.logger.critical(msg)
            raise ValueError(msg)

        leadership_team = self.teams["leadership"]
        lines.append("# Default owner of the teams directory is the tech director")
        lines.append(f"teams @{leadership_team.maintainers[0]}")
        lines.append("")

        # Assign the  maintainers of the team as the codeowners of the team's file
        # Sort the teams to prevent changes to the codeowners file due to ordering
        lines.append(
            "# The codeowners of each team file are the maintainers of the team\n"
        )
        for team in sorted(self.teams.values(), key=lambda x: x.slug):
            codeowners_pattern = f"teams/{team.slug}.toml"
            for maintainer in sorted(team.maintainers):
                codeowners_pattern += f" @{maintainer}"
            lines.append(codeowners_pattern)
            lines.append("")

        return "\n".join(lines)

    def get_current_codeowners_file(
        self, repo: Repository
    ) -> tuple[str | None, str | None]:
        """Get the current codeowners file from the repository."""
        sha, current_content = None, None
        # Get the contents of the codeowners file
        try:
            contents = repo.get_contents(".github/CODEOWNERS")
        except Exception:
            self.logger.exception("Failed to get the current codeowners file")
            return current_content, sha

        # Error if the contents is a list (i.e. a directory)
        if isinstance(contents, list):
            msg = (
                "Expected '.github/CODEOWNERS' to be a single file, "
                "but got a directory."
            )
            self.logger.error(msg)
            return None, None

        # Return the sha and content of the codeowners file
        sha = contents.sha
        current_content = contents.decoded_content.decode("utf-8")
        return current_content, sha
