# Issue Tracker: GitHub

Issues and PRDs for this repo live as GitHub issues in
`baicie/tunnel-mcp`. Use the `gh` CLI for issue operations when it is
available.

## Conventions

- Create an issue with `gh issue create --title "..." --body "..."`.
- Read an issue with `gh issue view <number> --comments`.
- List issues with `gh issue list --state open --json number,title,labels`.
- Comment with `gh issue comment <number> --body "..."`.
- Apply labels with `gh issue edit <number> --add-label "..."`.
- Remove labels with `gh issue edit <number> --remove-label "..."`.
- Close with `gh issue close <number> --comment "..."`.

Infer the repo from `git remote -v`; `gh` does this automatically inside the
clone.

## When A Skill Says "Publish To The Issue Tracker"

Create a GitHub issue.

## When A Skill Says "Fetch The Relevant Ticket"

Run:

```bash
gh issue view <number> --comments
```
