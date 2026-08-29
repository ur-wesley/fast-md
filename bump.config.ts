/** @type {import('bumpp').VersionBumpOptions} */
export default {
  // Target version tracking files
  files: [
    'package.json',
    'Cargo.toml',
  ],

  // Conventional commit message (v%s replaces %s with new version)
  commit: 'chore(release): v%s',

  // Git tag (true defaults to v<version>, or use 'v%s')
  tag: true,

  // Push git commit and tag
  push: true,

  // Include all modified files in the release commit (such as updated Cargo.lock)
  all: true,

  // Prompt for confirmation before applying changes
  confirm: true,

  // Synchronize Cargo.lock and verify workspace integrity before finalizing commit
  execute: 'cargo check',
}
