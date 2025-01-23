# Chive Format Specification

chives are TOML formatted plaintext

- `.FILENAME.chive`: sidecar for a specific file. If this exists, parent `.chive`s inlines should be removed. 
-`.DIRNAME.chive`: less recommended, a sidecar for a specific directory. Has the same effect as `.chive` inside the directory.

- `.chive`: generally describes files, it lives alongside them at the root of a directory.
  - for files w/out corresponding sidecar, the sidecar can be inlined, else list each found `.FILENAME.chive`.

- `FILENAME.chive`: a single "file" (tarball/directory); a binary and chive meta pair. This should be the smallest unit of a normal file; can be recursive but is ideally flat.
  - the chive meta should still be `.FILENAME.chive`
