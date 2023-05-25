# unitypackage_util

Requires `barcoderdev/FBX2glTF` to extract FBX in GLTF(glb) format.  Place the binary in the same folder, or in PATH.

---

```bash
Usage: unitypackage_util <PACKAGE> <COMMAND>

Commands:
  debug    Find source of dump crashes
  info     Show package info
  name     Display path from guid/pathname file
  dump     Dump package contents
  list     List package contents
  extract  Extract package file
  xx-hash  Calculate xxhash 64 of string
  help     Print this message or the help of the given subcommand(s)

Arguments:
  <PACKAGE>  Unity Package (Tar, TarGz, or Folder)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

---

```bash
Extract package file

Usage: unitypackage_util <PACKAGE> extract [OPTIONS] <GUID>

Arguments:
  <GUID>  

Options:
  -o, --output-file <OUTPUT_FILE>  Extract to file
  -m, --meta                       Extract /asset.meta file instead of /asset
  -j, --json                       Process yaml to json
  -p, --pretty                     Pretty Print JSON
  -f, --fbx2gltf                   Convert FBX to GLTF
  -b, --base64                     Base64 encode output
  -h, --help                       Print help
```
