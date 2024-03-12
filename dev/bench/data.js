window.BENCHMARK_DATA = {
  "lastUpdate": 1710204094853,
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
          "id": "a6b3407e2a31f6309db91e2b7eb9ef6e49c9f32d",
          "message": "Merge pull request #422 from nuttycom/test_only_random_merklehashorchard\n\nAdd a `MerkleHashOrchard::random` function under the `test-dependencies` feature.",
          "timestamp": "2024-03-11T18:26:44-06:00",
          "tree_id": "366987999a60f844b31252306bdb41e45b4c668e",
          "url": "https://github.com/zcash/orchard/commit/a6b3407e2a31f6309db91e2b7eb9ef6e49c9f32d"
        },
        "date": 1710204092995,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2911583279,
            "range": "± 254912600",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2898955434,
            "range": "± 7784756",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4170389356,
            "range": "± 24441025",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5428527892,
            "range": "± 22084542",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25328446,
            "range": "± 514312",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25655324,
            "range": "± 1256765",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28336834,
            "range": "± 531610",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31831547,
            "range": "± 435718",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1528024,
            "range": "± 12905",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 127658,
            "range": "± 366",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1524183,
            "range": "± 36034",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1354445299,
            "range": "± 872397",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16120226,
            "range": "± 45783",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2171530,
            "range": "± 10873",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16095731,
            "range": "± 51430",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2134778,
            "range": "± 6858",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80594235,
            "range": "± 190606",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10811798,
            "range": "± 31471",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80494683,
            "range": "± 165908",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10625927,
            "range": "± 30866",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 161187602,
            "range": "± 1265873",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21604749,
            "range": "± 44540",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 160886232,
            "range": "± 221365",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21243702,
            "range": "± 688358",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 466076,
            "range": "± 12139",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 500827,
            "range": "± 1954",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}