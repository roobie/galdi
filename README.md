# galdi

**Ever run a sketchy installer, deploy script, or build tool and wonder "what the heck did this thing just touch on my filesystem?"** Galdi has your back.

It's a fast, no-nonsense tool that snapshots your filesystem metadata (paths, sizes, checksums, timestamps) and lets you diff before/after states. Think `git status` but for your entire filesystem—except it actually tells you what changed, where, and how.

## Why use galdi?

**1. Debug build systems and installers**
Your build tool just created 47 files in random locations across your system. Which ones? Where? Galdi shows you exactly what appeared, disappeared, or changed—no more hunting through logs or guessing.

**2. Audit deployments and migrations**
Before you push that deployment script to production, run it on staging with galdi snapshots. You'll know if it's touching files it shouldn't be, missing expected updates, or leaving orphaned config files behind.

**3. Catch filesystem corruption early**
Regular snapshots with content checksums let you detect bit rot, accidental modifications, or suspicious changes in your important directories. Great for archives, backups, or any data you want to verify hasn't silently changed.

## Design goals
- **Fast** – Parallel scanning with optimized hashing (Blake3, XXH3, SHA256)
- **Reliable** – Deterministic output, robust error handling, extensive test coverage
- **Simple** – Clean JSON output, pipe-friendly, zero config needed
- **Cross-platform** – Works on Linux, macOS, Windows

## Quick start

```bash
# Capture a baseline snapshot before running something
galdi snapshot /some/path > before.json

# Run your build, installer, deploy script, whatever
./mystery-script.sh /some/path

# See exactly what changed
galdi diff /some/path before.json > changes.json

# Or just print it to your terminal in human-readable format
galdi diff /some/path before.json --human
```

## Output format

Galdi outputs structured JSON that's easy to parse, pipe to other tools, or just read directly. Each snapshot includes complete metadata for every file and directory.

### Snapshot output

`cat before.json`
```json
{
  "$plumbah": {
    "version": "1.0",
    "status": "ok",
    "meta": {
      "idempotent": true,
      "mutates": false,
      "safe": true,
      "deterministic": false,
      "plumbah_level": 2,
      "execution_time_ms": 5,
      "tool": "galdi_snapshot",
      "tool_version": "0.3.2",
      "timestamp": "2026-01-28T00:41:10.316825899Z"
    }
  },
  "version": "1.0",
  "root": ".",
  "checksum_algorithm": "xxh3_64",
  "count": 60,
  "entries": [
    {
      "path": "",
      "type": "directory",
      "size": 4096,
      "mode": "775",
      "mtime": "2026-01-28T00:41:09.192041753Z",
      "checksum": null,
      "target": null
    },
    {
      "path": "Cargo.lock",
      "type": "file",
      "size": 36808,
      "mode": "664",
      "mtime": "2026-01-28T00:25:12.337435754Z",
      "checksum": "xxh3_64:3e9a27e741478f22",
      "target": null
    },
    {
      "path": "Cargo.toml",
      "type": "file",
      "size": 202,
      "mode": "664",
      "mtime": "2026-01-28T00:23:46.370629738Z",
      "checksum": "xxh3_64:1b03302d1eb8e350",
      "target": null
    },
    {
      "path": "LICENSE",
      "type": "file",
      "size": 11357,
      "mode": "664",
      "mtime": "2026-01-28T00:17:58.871420133Z",
      "checksum": "xxh3_64:af8d6c471266dc43",
      "target": null
    },
    {
      "path": "examples",
      "type": "directory",
      "size": 4096,
      "mode": "775",
      "mtime": "2026-01-28T00:40:43.520402397Z",
      "checksum": null,
      "target": null
    },
    {
      "path": "examples/snapshot.json",
      "type": "file",
      "size": 0,
      "mode": "664",
      "mtime": "2026-01-28T00:41:10.300026188Z",
      "checksum": "xxh3_64:2d06800538d394c2",
      "target": null
    },
    {
      "path": "galdi",
      "type": "directory",
      "size": 4096,
      "mode": "775",
      "mtime": "2026-01-28T00:24:00.930427712Z",
      "checksum": null,
      "target": null
    },
    {
      "path": "galdi/Cargo.toml",
      "type": "file",
      "size": 764,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.930427712Z",
      "checksum": "xxh3_64:ebc6eb4f04326af0",
      "target": null
    },
    {
      "path": "galdi/src",
      "type": "directory",
      "size": 4096,
      "mode": "775",
      "mtime": "2026-01-28T00:24:00.930427712Z",
      "checksum": null,
      "target": null
    },
    {
      "path": "galdi/src/cli.rs",
      "type": "file",
      "size": 5725,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.930427712Z",
      "checksum": "xxh3_64:e0159bf7302a031b",
      "target": null
    },
    {
      "path": "galdi/src/main.rs",
      "type": "file",
      "size": 2426,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.930427712Z",
      "checksum": "xxh3_64:8ea1a7563c495c01",
      "target": null
    },
    {
      "path": "galdi/src/mcp.rs",
      "type": "file",
      "size": 7021,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.930427712Z",
      "checksum": "xxh3_64:6ea737fe7335d96c",
      "target": null
    },
    {
      "path": "galdi_core",
      "type": "directory",
      "size": 4096,
      "mode": "775",
      "mtime": "2026-01-28T00:24:56.541655338Z",
      "checksum": null,
      "target": null
    },
    {
      "path": "galdi_core/Cargo.toml",
      "type": "file",
      "size": 712,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.930427712Z",
      "checksum": "xxh3_64:11e307039a188e39",
      "target": null
    },
    {
      "path": "galdi_core/src",
      "type": "directory",
      "size": 4096,
      "mode": "775",
      "mtime": "2026-01-28T00:24:00.934427656Z",
      "checksum": null,
      "target": null
    },
    {
      "path": "galdi_core/src/checksum.rs",
      "type": "file",
      "size": 6200,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.930427712Z",
      "checksum": "xxh3_64:b3e717f4baac32fa",
      "target": null
    },
    {
      "path": "galdi_core/src/diff.rs",
      "type": "file",
      "size": 1257,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.930427712Z",
      "checksum": "xxh3_64:bdc5ca88285d67c4",
      "target": null
    },
    {
      "path": "galdi_core/src/error.rs",
      "type": "file",
      "size": 7480,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.934427656Z",
      "checksum": "xxh3_64:471cd950beee37f9",
      "target": null
    },
    {
      "path": "galdi_core/src/fs_scan.rs",
      "type": "file",
      "size": 8962,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.934427656Z",
      "checksum": "xxh3_64:e26cd7224de9e2fc",
      "target": null
    },
    {
      "path": "galdi_core/src/lib.rs",
      "type": "file",
      "size": 234,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.934427656Z",
      "checksum": "xxh3_64:80882510fba76928",
      "target": null
    },
    {
      "path": "galdi_core/src/plumbah.rs",
      "type": "file",
      "size": 12631,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.934427656Z",
      "checksum": "xxh3_64:a63a705d183b6532",
      "target": null
    },
    {
      "path": "galdi_core/src/snapshot.rs",
      "type": "file",
      "size": 16643,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.934427656Z",
      "checksum": "xxh3_64:e6973eef907a1b99",
      "target": null
    },
    {
      "path": "galdi_core/tests",
      "type": "directory",
      "size": 4096,
      "mode": "775",
      "mtime": "2026-01-28T00:24:00.934427656Z",
      "checksum": null,
      "target": null
    },
    {
      "path": "galdi_core/tests/common",
      "type": "directory",
      "size": 4096,
      "mode": "775",
      "mtime": "2026-01-28T00:24:00.934427656Z",
      "checksum": null,
      "target": null
    },
    {
      "path": "galdi_core/tests/common/assertions.rs",
      "type": "file",
      "size": 4740,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.934427656Z",
      "checksum": "xxh3_64:a2a1053ab56c3c98",
      "target": null
    },
    {
      "path": "galdi_core/tests/common/fixtures.rs",
      "type": "file",
      "size": 3350,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.934427656Z",
      "checksum": "xxh3_64:90f415df46a58843",
      "target": null
    },
    {
      "path": "galdi_core/tests/common/generators.rs",
      "type": "file",
      "size": 8059,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.934427656Z",
      "checksum": "xxh3_64:f47f118fecb3fe2f",
      "target": null
    },
    {
      "path": "galdi_core/tests/common/mod.rs",
      "type": "file",
      "size": 252,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.934427656Z",
      "checksum": "xxh3_64:8ccca024654f6d7c",
      "target": null
    },
    {
      "path": "galdi_core/tests/common/platform.rs",
      "type": "file",
      "size": 1977,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.934427656Z",
      "checksum": "xxh3_64:1717a8aa622878bf",
      "target": null
    },
    {
      "path": "galdi_core/tests/integration_checksum.rs",
      "type": "file",
      "size": 9923,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.934427656Z",
      "checksum": "xxh3_64:09862b0402d9e05f",
      "target": null
    },
    {
      "path": "galdi_core/tests/property_checksum.rs",
      "type": "file",
      "size": 7032,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.934427656Z",
      "checksum": "xxh3_64:f89f7bad774dcf1e",
      "target": null
    },
    {
      "path": "galdi_core/tests/property_plumbah.rs",
      "type": "file",
      "size": 6868,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.934427656Z",
      "checksum": "xxh3_64:9f2a98e2f5eed362",
      "target": null
    },
    {
      "path": "galdi_core/tests/property_scanner.rs",
      "type": "file",
      "size": 7276,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.934427656Z",
      "checksum": "xxh3_64:cc70e4a4a47b4f27",
      "target": null
    },
    {
      "path": "galdi_core/tests/property_snapshot.rs",
      "type": "file",
      "size": 5197,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.934427656Z",
      "checksum": "xxh3_64:75543745cde46b9d",
      "target": null
    },
    {
      "path": "galdi_diff",
      "type": "directory",
      "size": 4096,
      "mode": "775",
      "mtime": "2026-01-28T00:24:00.938427601Z",
      "checksum": null,
      "target": null
    },
    {
      "path": "galdi_diff/Cargo.toml",
      "type": "file",
      "size": 764,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.934427656Z",
      "checksum": "xxh3_64:35ca51d4e6f0b6c3",
      "target": null
    },
    {
      "path": "galdi_diff/examples",
      "type": "directory",
      "size": 4096,
      "mode": "775",
      "mtime": "2026-01-28T00:24:00.934427656Z",
      "checksum": null,
      "target": null
    },
    {
      "path": "galdi_diff/examples/diff.changes.json",
      "type": "file",
      "size": 2390,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.934427656Z",
      "checksum": "xxh3_64:4ca7bf1fa6953832",
      "target": null
    },
    {
      "path": "galdi_diff/examples/diff.identical.json",
      "type": "file",
      "size": 487,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.934427656Z",
      "checksum": "xxh3_64:d1780a7146a10213",
      "target": null
    },
    {
      "path": "galdi_diff/src",
      "type": "directory",
      "size": 4096,
      "mode": "775",
      "mtime": "2026-01-28T00:24:00.938427601Z",
      "checksum": null,
      "target": null
    },
    {
      "path": "galdi_diff/src/app.rs",
      "type": "file",
      "size": 6203,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.938427601Z",
      "checksum": "xxh3_64:82277dd06c338876",
      "target": null
    },
    {
      "path": "galdi_diff/src/cli.rs",
      "type": "file",
      "size": 2375,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.938427601Z",
      "checksum": "xxh3_64:cc22a11431f2dd5b",
      "target": null
    },
    {
      "path": "galdi_diff/src/diff.rs",
      "type": "file",
      "size": 4722,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.938427601Z",
      "checksum": "xxh3_64:704cc838af469bb2",
      "target": null
    },
    {
      "path": "galdi_diff/src/lib.rs",
      "type": "file",
      "size": 241,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.938427601Z",
      "checksum": "xxh3_64:0af2e7e5b8a38ce3",
      "target": null
    },
    {
      "path": "galdi_diff/src/main.rs",
      "type": "file",
      "size": 785,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.938427601Z",
      "checksum": "xxh3_64:76d5f0a2c1992564",
      "target": null
    },
    {
      "path": "galdi_diff/tests",
      "type": "directory",
      "size": 4096,
      "mode": "775",
      "mtime": "2026-01-28T00:24:00.938427601Z",
      "checksum": null,
      "target": null
    },
    {
      "path": "galdi_diff/tests/property_diff.rs",
      "type": "file",
      "size": 10891,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.938427601Z",
      "checksum": "xxh3_64:85bcaf7e8f0ccc57",
      "target": null
    },
    {
      "path": "galdi_snapshot",
      "type": "directory",
      "size": 4096,
      "mode": "775",
      "mtime": "2026-01-28T00:39:45.861212282Z",
      "checksum": null,
      "target": null
    },
    {
      "path": "galdi_snapshot/Cargo.toml",
      "type": "file",
      "size": 1086,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.938427601Z",
      "checksum": "xxh3_64:865c59b7c098c867",
      "target": null
    },
    {
      "path": "galdi_snapshot/sink",
      "type": "directory",
      "size": 4096,
      "mode": "775",
      "mtime": "2026-01-28T00:24:00.938427601Z",
      "checksum": null,
      "target": null
    },
    {
      "path": "galdi_snapshot/sink/jaq.rs",
      "type": "file",
      "size": 1047,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.938427601Z",
      "checksum": "xxh3_64:56586221c6e1b7b3",
      "target": null
    },
    {
      "path": "galdi_snapshot/src",
      "type": "directory",
      "size": 4096,
      "mode": "775",
      "mtime": "2026-01-28T00:24:00.938427601Z",
      "checksum": null,
      "target": null
    },
    {
      "path": "galdi_snapshot/src/app.rs",
      "type": "file",
      "size": 4909,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.938427601Z",
      "checksum": "xxh3_64:9cd43a274df25d28",
      "target": null
    },
    {
      "path": "galdi_snapshot/src/cli.rs",
      "type": "file",
      "size": 2236,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.938427601Z",
      "checksum": "xxh3_64:8d43d02ebb822ea9",
      "target": null
    },
    {
      "path": "galdi_snapshot/src/lib.rs",
      "type": "file",
      "size": 219,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.938427601Z",
      "checksum": "xxh3_64:02f7181a12dd4359",
      "target": null
    },
    {
      "path": "galdi_snapshot/src/main.rs",
      "type": "file",
      "size": 856,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.938427601Z",
      "checksum": "xxh3_64:65f99b08bdd90833",
      "target": null
    },
    {
      "path": "galdi_snapshot/src/output.rs",
      "type": "file",
      "size": 4074,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.938427601Z",
      "checksum": "xxh3_64:9851edb2371fce92",
      "target": null
    },
    {
      "path": "galdi_snapshot/tests",
      "type": "directory",
      "size": 4096,
      "mode": "775",
      "mtime": "2026-01-28T00:24:00.942427545Z",
      "checksum": null,
      "target": null
    },
    {
      "path": "galdi_snapshot/tests/integration_jsonl.rs",
      "type": "file",
      "size": 17016,
      "mode": "664",
      "mtime": "2026-01-28T00:24:00.942427545Z",
      "checksum": "xxh3_64:8a2820b6205e05bd",
      "target": null
    },
    {
      "path": "mise.toml",
      "type": "file",
      "size": 1230,
      "mode": "644",
      "mtime": "2026-01-28T00:30:47.724757169Z",
      "checksum": "xxh3_64:17a1ed3ec409b4ec",
      "target": null
    }
  ]
}
```

### Diff output

The diff shows you added, removed, and modified files with a handy summary and detailed change information.

`cat changes.json`

```json
{
  "$plumbah": {
    "version": "1.0",
    "status": "ok",
    "meta": {
      "idempotent": true,
      "mutates": false,
      "safe": true,
      "deterministic": true,
      "plumbah_level": 2,
      "execution_time_ms": 0,
      "tool": "galdi_diff",
      "tool_version": "0.3.2",
      "timestamp": "2026-01-28T00:48:39.156022890Z"
    }
  },
  "identical": false,
  "summary": {
    "added": 3,
    "removed": 4,
    "modified": 4,
    "unchanged": 53
  },
  "differences": [
    {
      "path": "",
      "change_type": "modified",
      "changes": [
        "mtime"
      ],
      "source": {
        "path": "",
        "type": "directory",
        "size": 4096,
        "mode": "775",
        "mtime": "2026-01-28T00:48:19.403078814Z",
        "checksum": null,
        "target": null
      },
      "target": {
        "path": "",
        "type": "directory",
        "size": 4096,
        "mode": "775",
        "mtime": "2026-01-28T00:41:09.192041753Z",
        "checksum": null,
        "target": null
      }
    },
    {
      "path": "README.md",
      "change_type": "removed",
      "source": {
        "path": "README.md",
        "type": "file",
        "size": 14700,
        "mode": "664",
        "mtime": "2026-01-28T00:48:19.403078814Z",
        "checksum": "xxh3_64:f0aea9c4f8d85437",
        "target": null
      },
      "target": null
    },
    {
      "path": "examples",
      "change_type": "modified",
      "changes": [
        "mtime"
      ],
      "source": {
        "path": "examples",
        "type": "directory",
        "size": 4096,
        "mode": "775",
        "mtime": "2026-01-28T00:48:39.134688283Z",
        "checksum": null,
        "target": null
      },
      "target": {
        "path": "examples",
        "type": "directory",
        "size": 4096,
        "mode": "775",
        "mtime": "2026-01-28T00:40:43.520402397Z",
        "checksum": null,
        "target": null
      }
    },
    {
      "path": "examples/diff.changes.json",
      "change_type": "removed",
      "source": {
        "path": "examples/diff.changes.json",
        "type": "file",
        "size": 2390,
        "mode": "664",
        "mtime": "2026-01-28T00:24:00.934427656Z",
        "checksum": "xxh3_64:4ca7bf1fa6953832",
        "target": null
      },
      "target": null
    },
    {
      "path": "examples/diff.identical.json",
      "change_type": "removed",
      "source": {
        "path": "examples/diff.identical.json",
        "type": "file",
        "size": 487,
        "mode": "664",
        "mtime": "2026-01-28T00:24:00.934427656Z",
        "checksum": "xxh3_64:d1780a7146a10213",
        "target": null
      },
      "target": null
    },
    {
      "path": "examples/diff.json",
      "change_type": "removed",
      "source": {
        "path": "examples/diff.json",
        "type": "file",
        "size": 0,
        "mode": "664",
        "mtime": "2026-01-28T00:48:39.134688283Z",
        "checksum": "xxh3_64:2d06800538d394c2",
        "target": null
      },
      "target": null
    },
    {
      "path": "examples/snapshot.json",
      "change_type": "modified",
      "changes": [
        "content",
        "mtime",
        "size"
      ],
      "source": {
        "path": "examples/snapshot.json",
        "type": "file",
        "size": 14126,
        "mode": "664",
        "mtime": "2026-01-28T00:41:10.312026018Z",
        "checksum": "xxh3_64:41da5fb48de14e01",
        "target": null
      },
      "target": {
        "path": "examples/snapshot.json",
        "type": "file",
        "size": 0,
        "mode": "664",
        "mtime": "2026-01-28T00:41:10.300026188Z",
        "checksum": "xxh3_64:2d06800538d394c2",
        "target": null
      }
    },
    {
      "path": "galdi_diff",
      "change_type": "modified",
      "changes": [
        "mtime"
      ],
      "source": {
        "path": "galdi_diff",
        "type": "directory",
        "size": 4096,
        "mode": "775",
        "mtime": "2026-01-28T00:42:05.399252009Z",
        "checksum": null,
        "target": null
      },
      "target": {
        "path": "galdi_diff",
        "type": "directory",
        "size": 4096,
        "mode": "775",
        "mtime": "2026-01-28T00:24:00.938427601Z",
        "checksum": null,
        "target": null
      }
    },
    {
      "path": "galdi_diff/examples",
      "change_type": "added",
      "source": null,
      "target": {
        "path": "galdi_diff/examples",
        "type": "directory",
        "size": 4096,
        "mode": "775",
        "mtime": "2026-01-28T00:24:00.934427656Z",
        "checksum": null,
        "target": null
      }
    },
    {
      "path": "galdi_diff/examples/diff.changes.json",
      "change_type": "added",
      "source": null,
      "target": {
        "path": "galdi_diff/examples/diff.changes.json",
        "type": "file",
        "size": 2390,
        "mode": "664",
        "mtime": "2026-01-28T00:24:00.934427656Z",
        "checksum": "xxh3_64:4ca7bf1fa6953832",
        "target": null
      }
    },
    {
      "path": "galdi_diff/examples/diff.identical.json",
      "change_type": "added",
      "source": null,
      "target": {
        "path": "galdi_diff/examples/diff.identical.json",
        "type": "file",
        "size": 487,
        "mode": "664",
        "mtime": "2026-01-28T00:24:00.934427656Z",
        "checksum": "xxh3_64:d1780a7146a10213",
        "target": null
      }
    }
  ]
}
```

## Hash algorithms

By default, galdi uses `xxh3_64` for checksums—it's blazing fast but not cryptographically secure. That's fine for detecting accidental changes or bit rot.

If you need cryptographic verification (like detecting malicious tampering), use `blake3` or `sha256`:

```bash
# Use Blake3 (fast + cryptographically secure)
galdi snapshot /path --checksum blake3 > snapshot.json

# Use SHA256 (slower but widely trusted)
galdi snapshot /path --checksum sha256 > snapshot.json
```

**Performance reference:** On a typical SSD, xxh3_64 can hash several GB/s, blake3 does ~1-2 GB/s, and sha256 is around 500 MB/s. Pick based on your threat model, not on vibes.
