# skill-installer Specification Delta

## MODIFIED Requirements

### Requirement: Lock File Update
`install` MUST accept both legacy format (where `skills` is an array) and new format (where `skills` is a map) when reading the lock file. When legacy format is detected, `install` MUST automatically convert it to the new format during load.

#### Scenario: Automatic migration from legacy lock file format
Given `~/.agents/.skill-lock.json` exists with the following content:
```json
{
  "skills": [
    {"name": "skill-a", "path": "/path/to/a", "source_type": "github"},
    {"name": "skill-b", "path": "", "source_type": "github"}
  ]
}
```
When `install` loads the lock file.
Then `skill-a` is imported as a new format entry, and `skill-b` is excluded because its `path` is empty.
And the lock file is treated as new format (with `version` and `skills` map).

#### Scenario: Backward compatibility with new format lock file
Given a lock file where `skills` is stored in map format.
When `install` loads the lock file.
Then it succeeds as usual without any behavioral change.
