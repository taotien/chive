philosophy
  - make file management as human friendly as possible
  - automation maxxing
  - never modify original binary data except binary data

mods
  - tagger/readonly-meta (sidecars)
    - .chive hidden file for a whole dir
  - metadata manager
    - filename.chive 
  - filesystem?
    - flatly store all files
    - quickly sort files into "directories" based on scenario, exposed to fs using links or namespaces
    - this is both for and against philosophy of chive, in that it maximizes flexibility, but makes everything less human-friendly

tagger
  - check filenames in current dir
    - check checksums in current dir

renamer/mover
  - rename based on tags
  - detect common patterns (regex?)

linker
  - for safer changes while chive is unstable?

stripper
  - decouple metadata from binary data  

hasher
  - blake3

ui
  - nushell plugin
  - tui
  - gui
  - shell hooks

daemon
file format
file system
