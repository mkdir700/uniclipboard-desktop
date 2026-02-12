# Learnings

- When refactoring UI components, always update tests to reflect the new structure and interactions.
- Be careful with `git add` and ensure only intended files are staged.
- Use `git commit --amend` to fix accidental commits.
- `AnimatePresence` and `motion.div` from `framer-motion` work well for accordion animations.
- `e.stopPropagation()` is crucial for nested interactive elements like buttons inside a clickable row.
- `DevicesPage` simplification requires removing section composition points first (`DeviceHeader`, inline pairing requests, `CurrentDevice`) before polishing child visuals.
- In this repo, targeted Vitest runs should use `--dir src` to avoid picking duplicated tests from `.worktrees/` during local verification.
