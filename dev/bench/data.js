window.BENCHMARK_DATA = {
  "lastUpdate": 1710874090397,
  "repoUrl": "https://github.com/zcash/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "kris@nutty.land",
            "name": "Kris Nuttycombe",
            "username": "nuttycom"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "33474bdbfd7268e1f84718078d47f63d01a879d5",
          "message": "Merge pull request #423 from nuttycom/merkle_hash_orchard_dist\n\nAdd `impl Distribution<MerkleHashOrchard> for Standard` for testing.",
          "timestamp": "2024-03-19T12:34:27-06:00",
          "tree_id": "c4846a51b1e38e3d8bf9608ad11c3c87ebab51c3",
          "url": "https://github.com/zcash/orchard/commit/33474bdbfd7268e1f84718078d47f63d01a879d5"
        },
        "date": 1710874089347,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2900932090,
            "range": "± 36366045",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2896728045,
            "range": "± 22498491",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4147010848,
            "range": "± 23602885",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5391503582,
            "range": "± 21484106",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 24842896,
            "range": "± 593738",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25029959,
            "range": "± 527627",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28068912,
            "range": "± 410813",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31552679,
            "range": "± 387191",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1531258,
            "range": "± 12532",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 127949,
            "range": "± 456",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1526667,
            "range": "± 3594",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1361891152,
            "range": "± 4305617",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16162976,
            "range": "± 271470",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2177675,
            "range": "± 11700",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16119257,
            "range": "± 30200",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2144281,
            "range": "± 1552",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80738150,
            "range": "± 232075",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10832683,
            "range": "± 49808",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80541828,
            "range": "± 255042",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10656511,
            "range": "± 78035",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 161509770,
            "range": "± 381607",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21637894,
            "range": "± 101754",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 161035862,
            "range": "± 102738",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21290115,
            "range": "± 874743",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 468254,
            "range": "± 4358",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 500571,
            "range": "± 1205",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}